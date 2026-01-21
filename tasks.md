
# ✅ **CREDIT CARD BALANCE APP — TASK LIST**

---

## 🚀 **PHASE 1 — PROJECT SETUP (FOUNDATION)**

### **1.1 Create Project**

* [X] Create new Rust project
* [X] Initialize Git repository
* [X] Add dependencies in `Cargo.toml`:

  * [d] axum
  * [X] tokio (full)
  * [X] sqlx (sqlite + macros + runtime-tokio-rustls)
  * [X] serde / serde_json
  * [X] uuid
  * [X] bcrypt or argon2
  * [X] jsonwebtoken
  * [X] dotenv (optional)

### **1.2 Basic Axum Server**

* [X] Create `main.rs`
* [X] Setup Axum router
* [X] Setup SQLite connection pool
* [X] Start server on `127.0.0.1:3000`
* [ ] Add health check endpoint:

  * [ ] `GET /health → "OK"`

---

## 🗄️ **PHASE 2 — DATABASE & MIGRATIONS**

### **2.1 SQLite Setup**

* [ ] Create `db.sqlite`
* [ ] Install `sqlx-cli`
* [ ] Create `migrations/` folder

### **2.2 Create Tables (Migration #1)**

* [ ] Create `users` table
* [ ] Create `cards` table
* [ ] Create `card_events` table
* [ ] Create `card_running_state` table
* [ ] Run `sqlx migrate run`

---

## 🔐 **PHASE 3 — AUTHENTICATION (JWT)**

### **3.1 User Registration**

* [ ] Create endpoint: `POST /login/create/`
* [ ] Parse JSON body (`user_name`, `password`)
* [ ] Hash password
* [ ] Generate `user_id = UUID`
* [ ] Insert into `users` table
* [ ] Return `201 Created`

### **3.2 User Login**

* [ ] Create endpoint: `POST /login/login`
* [ ] Fetch user by `user_name`
* [ ] Verify password hash
* [ ] Generate JWT with:

  * [ ] user_id
  * [ ] user_role
* [ ] Return JWT token in response

### **3.3 Auth Middleware**

* [ ] Create Axum middleware
* [ ] Read `Authorization: Bearer <token>`
* [ ] Validate JWT
* [ ] Extract `user_id`
* [ ] Pass `user_id` to handlers via `Extension`

---

## 💳 **PHASE 4 — CARD APIs**

### **4.1 Create Card**

* [ ] Endpoint: `POST /card/create/`
* [ ] Extract `user_id` from JWT
* [ ] Generate `card_id = UUID`
* [ ] Insert into `cards` table
* [ ] Initialize `card_running_state` with:

  * [ ] last_total_due = 0
  * [ ] last_delta = 0
* [ ] Return `201 Created`

### **4.2 Read Single Card**

* [ ] Endpoint: `GET /card/read/{card_id}`
* [ ] Fetch card from DB
* [ ] Validate card belongs to logged-in user
* [ ] Return card JSON

### **4.3 Update Card**

* [ ] Endpoint: `PUT /card/update/{card_id}`
* [ ] Validate ownership via `user_id`
* [ ] Update `card_name` and/or `card_bank`

### **4.4 Delete Card**

* [ ] Endpoint: `DELETE /card/delete/{card_id}`
* [ ] Delete from:

  * [ ] cards
  * [ ] card_running_state
  * [ ] (Optional) card_events

### **4.5 Get All Cards**

* [ ] Endpoint: `GET /card/all`
* [ ] Fetch all cards for user
* [ ] Return as JSON map (`card_1`, `card_2`, etc.)

---

## 🔁 **PHASE 5 — CARD TRANSACTIONS (CORE LOGIC)**

### **5.1 Add Card Event**

* [ ] Endpoint: `POST /card/event/{card_id}`
* [ ] Accept body: `{ "total_due_input": X }`
* [ ] Generate `transaction_id = UUID`
* [ ] Insert into `card_events`
* [ ] Run HOT update on `card_running_state`:

  * [ ] Calculate `last_delta`
  * [ ] Update `last_total_due`
  * [ ] Update `updated_at`
* [ ] Return `{ "last_delta": X }`

### **5.2 Get Running State**

* [ ] Endpoint: `GET /card/state/{card_id}`
* [ ] Fetch from `card_running_state`
* [ ] Return JSON with:

  * [ ] last_total_due
  * [ ] last_delta
  * [ ] updated_at

---

## 🧠 **PHASE 6 — BUSINESS LOGIC (YOUR ACTUAL APP GOAL)**

### **6.1 Transfer Suggestion**

* [ ] Endpoint: `GET /card/transfer-suggestion/{card_id}`
* [ ] Read `last_delta` from DB
* [ ] Return:

  * [ ] `transfer_amount = last_delta`
  * [ ] Human-readable message like:

    * “Move ₹X from savings to spending account”

---

## 🛡️ **PHASE 7 — EDGE CASES & VALIDATION**

* [ ] Reject negative `total_due_input`
* [ ] If `new_due < last_due` → return 400 error
* [ ] Validate all UUIDs
* [ ] Ensure users can only access their own cards
* [ ] Add basic rate limiting (optional)

---

## 🪵 **PHASE 8 — LOGGING & OBSERVABILITY**

* [ ] Add structured logging (`tracing` or `log`)
* [ ] Log:

  * [ ] Successful logins
  * [ ] Failed logins
  * [ ] Card creation
  * [ ] Transaction updates

---

## 🧑‍💻 **PHASE 9 — FRONTEND READINESS**

Ensure these APIs exist cleanly:

### Auth

* [ ] `POST /login/create/`
* [ ] `POST /login/login`

### Cards

* [ ] `POST /card/create/`
* [ ] `GET /card/read/{card_id}`
* [ ] `GET /card/all`
* [ ] `PUT /card/update/{card_id}`
* [ ] `DELETE /card/delete/{card_id}`

### Transactions

* [ ] `POST /card/event/{card_id}`
* [ ] `GET /card/state/{card_id}`
* [ ] `GET /card/transfer-suggestion/{card_id}`

---

## 🔮 **PHASE 10 — FUTURE IMPROVEMENTS (NICE TO HAVE)**

* [ ] Replace JWT with **PASETO**
* [ ] Add role-based access (admin vs user)
* [ ] Implement soft delete for cards
* [ ] Add audit logs
* [ ] Add tests for:

  * [ ] Login flow
  * [ ] Card creation
  * [ ] Transaction logic

---

