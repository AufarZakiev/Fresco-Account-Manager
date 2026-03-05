# FAM ↔ BOINC Client ↔ Project Server Integration Plan

## Current State

FAM has three actors:

```
┌──────────┐       AM RPC (XML)       ┌──────────────┐
│  BOINC   │ ──────────────────────── │     FAM      │
│  Client  │  every repeat_sec (600s) │    Server    │
└──────────┘                          └──────┬───────┘
                                             │  am_get_info / am_set_info
                                             │  create_account / lookup_account
                                             ▼
                                      ┌──────────────┐
                                      │   Project    │
                                      │   Server     │
                                      └──────────────┘
```

### What works

- User registers on FAM, enrolls projects via web UI
- BOINC client polls FAM (`/rpc.php`) — FAM replies with project list, resource shares, suspend/detach flags, global preferences
- FAM parses the client's AM request including per-project `account_key`
- `ProjectRpcClient` can call `create_account`, `lookup_account`, `am_get_info`, `am_set_info`, `get_project_config` on any project server
- `provision_project_account()` exists in `fam-core/src/provisioning.rs`

### What's broken

**`project_authenticator` is never populated.** The enroll endpoint creates a `user_projects` row with an empty authenticator. No code path fills it in.

Consequences:
- `fetch_user_accounts()` filters out rows where `project_authenticator = ''` → client never receives project attachments
- `get_project_prefs` returns `None` (empty auth → early return)
- `set_project_prefs` returns 412 (empty auth → precondition failed)
- The BOINC client sends `account_key` per project in every RPC, but `handle_rpc_inner()` ignores `request.projects` entirely

---

## Integration Strategy

Two complementary flows that cover all use cases.

### Flow A: Capture authenticators from BOINC client (primary, seamless)

The BOINC client already sends project authenticators in every AM RPC:

```xml
<acct_mgr_request>
  <authenticator>fam-user-token</authenticator>
  <host_cpid>CPID_123</host_cpid>
  <project>
    <url>https://einsteinathome.org/</url>
    <project_name>Einstein@Home</project_name>
    <account_key>abc123def456</account_key>
    <attached_via_acct_mgr>1</attached_via_acct_mgr>
  </project>
</acct_mgr_request>
```

**During `handle_rpc_inner()`, after authenticating the user:**

1. Iterate `request.projects`
2. For each project with a non-empty `account_key`:
   a. Match `project.url` against the `projects` table
   b. If a `user_projects` row exists for this user + project with empty authenticator → store `account_key`
   c. If no `user_projects` row exists but `attached_via_acct_mgr` is true → auto-create the row with the authenticator (the client is already attached via this AM, honor it)
   d. If `attached_via_acct_mgr` is false → the user manually attached this project outside FAM. Do NOT create a row. Optionally log it for the user to see in the web UI ("unmanaged projects").

**This requires zero user action.** The user enrolls a project in FAM web UI, tells their BOINC client to use FAM as account manager, and on the next RPC cycle the authenticator flows in automatically.

**Edge case — user enrolls on FAM but has no BOINC client yet:**
The authenticator stays empty until a client connects. The web UI should show "Waiting for client to connect" instead of the prefs form.

**Edge case — user has a client but hasn't attached the project yet:**
FAM replies with the project URL (once authenticator is obtained via Flow B below). The client will call `lookup_account` on the project server using the authenticator FAM provides, and attach automatically. On the next RPC cycle, the client sends back its `account_key` confirming attachment.

### Flow B: Provision via web UI (secondary, explicit)

For users who:
- Don't have a BOINC client running yet
- Want project prefs immediately without waiting for a client RPC cycle
- Need to create a new project account

**Two sub-flows:**

#### B1: Existing project account (lookup)

User provides their project email + password in FAM web UI.

1. Frontend sends `POST /api/user/projects/:id/provision` with `{ email, password }`
2. Backend computes `password_hash = md5(password + lowercase(email))` (BOINC convention)
3. Backend calls `lookup_account(project_url, email, password_hash)`
4. On success: stores authenticator in `user_projects`, returns ok
5. On `ERR_NOT_FOUND` (-136): returns error "Account not found on this project"
6. On `ERR_BAD_PASSWD` (-206): returns error "Incorrect password"

**The password is never stored.** Only the returned authenticator token is kept.

#### B2: New project account (create)

User clicks "Create account on this project" in FAM web UI.

1. Backend calls `get_project_config(project_url)` to check if creation is allowed
2. If `account_creation_disabled` → return error "This project requires manual registration at {home_url}"
3. Backend calls `create_account(project_url, email, password_hash, user_name)` using the user's FAM email/name and a generated password hash
4. On success: stores authenticator
5. On `-208` (WCG-style creation disabled): return error with project registration URL

**Important:** The password hash used for project account creation should be derived from the user's FAM password or a per-project generated secret. The user should understand that FAM is creating an account on their behalf.

---

## User Experience

### Scenario 1: New user, fresh setup

1. User registers on FAM web
2. Browses project catalog, enrolls in Einstein@Home and Rosetta
3. FAM creates `user_projects` rows (empty authenticators)
4. Web UI shows: "Enrolled — connect a BOINC client to start computing"
5. User installs BOINC client, sets FAM as account manager
6. Client contacts FAM → FAM has no authenticators yet, but knows the project URLs
7. **Problem:** FAM can't tell the client to attach without an authenticator

**Resolution options:**
- **Option 1:** At enrollment, immediately provision via `create_account` or `lookup_account`. Requires user to either provide project credentials or agree to auto-create.
- **Option 2 (recommended):** Show a "Link your project account" form (Flow B1) after enrollment. Once linked, the next AM RPC sends the authenticator to the client.

### Scenario 2: Existing BOINC user, already computing

1. User already has BOINC client with 5 projects attached
2. Registers on FAM, sets FAM as account manager in BOINC client
3. Client sends AM RPC with all 5 projects and their `account_key`s
4. FAM receives the RPC, authenticates user, sees 5 projects
5. **Flow A kicks in:** for each project, FAM auto-creates `user_projects` with the received authenticator
6. User opens FAM web — sees all 5 projects with full controls

This is the most common scenario and should Just Work.

### Scenario 3: User wants to change project preferences

1. User clicks "Preferences" on Einstein@Home in FAM web
2. FAM calls `am_get_info.php?account_key=X` → gets current `project_prefs` XML
3. Displays form (or raw XML editor)
4. User changes settings, clicks Save
5. FAM calls `am_set_info.php?account_key=X&project_prefs={xml}`
6. FAM sets `force_update = true` on the `user_projects` row
7. Next AM RPC: FAM sends `<update>1</update>` to the client
8. Client re-contacts the project scheduler → picks up new prefs

---

## Implementation Steps

### Step 1: Capture authenticators during AM RPC (Flow A)

**File: `crates/fam-server/src/routes/rpc.rs`**

After `upsert_host()` (step 3) and before `fetch_user_accounts()` (step 4), add:

```rust
// Capture project authenticators from client
sync_project_authenticators(&state.db, user.id, &request.projects).await;
```

New function `sync_project_authenticators`:
- For each `RequestProject` in the request:
  - Skip if `account_key` is `None` or empty
  - Normalize `project.url` (ensure trailing slash)
  - `UPDATE user_projects SET project_authenticator = $1 FROM projects WHERE ...`
    matching on `user_id`, `project.url`, and only if current authenticator is empty
  - If `attached_via_acct_mgr` and no `user_projects` row exists:
    find the project by URL, insert a new `user_projects` row with the authenticator

### Step 2: Add provision endpoint (Flow B1)

**File: `crates/fam-server/src/routes/api/user_projects.rs`**

New endpoint: `POST /api/user/projects/:id/provision`

```rust
#[derive(Deserialize)]
struct ProvisionRequest {
    email: String,
    password: String,
}
```

- Compute `md5(password + email.to_lowercase())`
- Call `lookup_account(project_url, email, hash)`
- Store returned authenticator
- Return `{ ok: true }` or error

### Step 3: Frontend status display

**File: `frontend/src/views/MyProjectsView.vue`**

Show different states per project:
- **Authenticator present:** full controls (prefs, suspend, resource share)
- **Authenticator empty:** "Link your account" button → opens provision modal (email + password form)
- **After linking:** refresh to show full controls

### Step 4: Optional — auto-create endpoint (Flow B2)

Lower priority. Only needed if users want to join projects they don't have accounts on.

`POST /api/user/projects/:id/create-account`
- Check `get_project_config` for `account_creation_disabled`
- Call `create_account`
- Store authenticator

---

## Security Considerations

### What FAM stores

| Data | Storage | Risk |
|------|---------|------|
| User password | bcrypt hash | Low — standard |
| FAM authenticator | Random token in `users.authenticator` | Low — revocable |
| Project authenticator | Plaintext in `user_projects.project_authenticator` | **Medium** — if DB is breached, attacker can impersonate users on all linked projects |
| User's project password | **Never stored** — used once during provisioning to call `lookup_account`, then discarded | None |

### What travels over the network

| Channel | Data | Protection |
|---------|------|------------|
| Browser ↔ FAM | Session cookie, prefs XML | HTTPS (TLS) |
| BOINC Client ↔ FAM | User authenticator, project `account_key`s | HTTPS (TLS) — BOINC client supports HTTPS for AM URLs |
| FAM ↔ Project Server | `account_key` in URL query params | HTTPS — most BOINC projects support HTTPS. **Query params may appear in server access logs** |

### Risks and mitigations

1. **Project authenticator in DB plaintext**
   - This is standard for BOINC account managers (BAM! does the same)
   - The authenticator is not a password — it's a bearer token specific to one project
   - Mitigation: DB access controls, encryption at rest, regular backups
   - Future: consider encrypting authenticators with a server-side key

2. **Authenticator in URL query params (FAM → project server)**
   - BOINC protocol design — `am_get_info.php?account_key=X` is how all AMs work
   - Risk: appears in project server access logs, CDN logs, proxy logs
   - Cannot be changed without BOINC protocol modifications
   - Mitigation: use HTTPS, which hides query params from intermediaries

3. **User provides project password during provisioning (Flow B1)**
   - Password is sent from browser to FAM over HTTPS
   - FAM computes `md5(password + email)` in memory, calls project server, discards immediately
   - **Must not log the password or password hash**
   - The md5 hash is a BOINC convention, not FAM's choice — all BOINC project auth uses this format

4. **Auto-enrollment from client RPC (Flow A)**
   - Only creates rows for projects where `attached_via_acct_mgr = true`
   - A malicious BOINC client could send fake `account_key`s → would be stored but wouldn't grant access to anything in FAM (only used for outbound calls to project servers)
   - Mitigation: the user must first authenticate to FAM with valid credentials. A compromised client can only affect its own user's data.

5. **Project account creation (Flow B2)**
   - Creates accounts using the user's FAM email — user should be informed
   - Some projects may have terms of service — FAM should show project registration URL as an alternative

---

## What the user needs to do

### Minimum (Scenario 2 — existing BOINC user):

1. Register on FAM website
2. In BOINC client: Account Manager → set FAM URL + login
3. Done — projects sync automatically on next RPC cycle

### If project prefs needed before client connects (Scenario 1):

1. Register on FAM website
2. Enroll in projects
3. Click "Link account" per project → enter project email + password (one-time)
4. Set preferences immediately
5. Later: connect BOINC client to FAM for actual computing

---

## Unmanaged projects

When the client reports projects with `attached_via_acct_mgr = false`, these are projects the user attached manually outside FAM. FAM should:

1. **Not create `user_projects` rows** for these
2. Optionally show them in the web UI as "Unmanaged" (read-only info from client RPC data: URL, name, resource share)
3. Offer an "Import to FAM" action that would store the `account_key` and create a managed row

---

## Summary

| Flow | Trigger | User action | Authenticator source |
|------|---------|-------------|---------------------|
| A — Client sync | BOINC client AM RPC | None (automatic) | `account_key` from client XML |
| B1 — Web provision | User clicks "Link account" | Enter project email + password | `lookup_account.php` response |
| B2 — Web create | User clicks "Create account" | Confirm creation | `create_account.php` response |
