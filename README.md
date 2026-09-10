# Capacity Connect

## Smart India Hackathon

- **PS ID:** 26075
- **Problem:** Capacity Connect — a digital capacity-building and learning management portal
- **Category:** Software
- **Theme:** Smart Education

## Overview

Capacity Connect is a role-based learning platform for trainees, trainers, and
administrators. It brings courses, lectures, assessments, resources, progress tracking,
certificates, competency mapping, and platform communication into one system.

## Features

### Trainees

- Sign up, enroll in courses, and complete lectures
- Track course progress and assessment performance
- Take timed MCQ assessments and view results
- Download learning resources
- Earn certificates for completed courses
- Save skills/interests and receive course-trainer recommendations
- View announcements, achievements, and learning updates

### Trainers

- Create courses and lectures
- Define course-required skills
- Create and manage assessments
- Upload course resources
- View enrollment, trainee progress, and assessment results

### Administrators

- View platform statistics
- Manage user roles
- Publish targeted announcements, notifications, achievements, and homepage content
- Edit or delete published content

## Technology

- **Frontend:** HTML, CSS, Vanilla JavaScript
- **Backend:** Rust, Axum, Tokio
- **Database:** SQLite with Rusqlite
- **Authentication:** JWT
- **Hosting:** GitHub Pages frontend + Render backend

```text
Users -> HTML/CSS/JS Frontend -> Rust Axum API -> SQLite
```

## Local Setup

### Backend

```powershell
Set-Location backend
cargo run
```

The API runs at `http://localhost:6969`.

### Frontend

Serve the `frontend` folder with a static server:

```powershell
npx serve frontend
```

Configure the backend origin in `frontend/js/auth.js`, `frontend/js/api.js`, and
`frontend/js/home.js`:

```javascript
window.CAPACITY_API_BASE = "https://your-backend.example.com";
```

Do not add `/api`; the frontend appends it automatically.

## Demo Account

```text
Email: admin@gmail.com
Password: 123456789
```

The backend seeds this account on first startup. Override it with:

```text
ADMIN_EMAIL=admin@gmail.com
ADMIN_PASSWORD=123456789
```

## Deployment Environment

Configure these variables on the backend host:

```text
HOST=0.0.0.0
PORT=<host-provided-port>
JWT_SECRET=<long-random-secret>
ADMIN_EMAIL=admin@gmail.com
ADMIN_PASSWORD=123456789
```

Keep `JWT_SECRET` and other backend secrets out of the frontend repository.

## Testing

From `backend/`:

```powershell
cargo check
cargo test
```

Tests cover authentication, roles, assessments, enrollment, progress, and resource
upload/download permissions.

## Demo Video

[Watch the Capacity Connect demo](https://drive.google.com/file/d/1b65Zaw9BDv5kYeiACpN6-fvRBEqATKm4/view?usp=sharing)
