# Capacity Connect

## Smart India Hackathon Submission

> **Project title:** Capacity Connect  
> **PS ID:** `26075`  
> **PS title:** `Participants are invited to design and develop *CAPACITY CONNECT A Digital Capacity Building and Learning Management Portal* to support organizational training, competency development, and knowledge sharing through a centralized web-based platform.`  
> **Category:** Software  
> **Theme:** `Smart Education`

## 1. Project Overview

Capacity Connect is a role-based learning and training platform that connects trainees,
trainers, and administrators in one place. It supports the complete learning flow:
discovering courses, enrolling, studying lectures, completing assessments, tracking
progress, accessing resources, and earning certificates.

The platform is designed to make skill development measurable and easier to manage:

- Trainees get personalized progress tracking and competency-based recommendations.
- Trainers create courses and assessments, upload resources, and monitor participation.
- Administrators manage users, publish platform updates, and manage learning content.

## 2. Problem Statement

Training programs often use disconnected tools for course material, assessments,
progress tracking, announcements, and learner performance. This makes it difficult for
trainees to understand their progress and for trainers or administrators to measure
learning outcomes.

Capacity Connect provides a unified platform for structured learning delivery,
performance monitoring, and competency-based course discovery.

## 3. Key Features

### Trainee

- Account signup and login
- Course discovery and enrollment
- Course-specific lectures and progress tracking
- Timed subject-wise MCQ assessments
- Automatic assessment scoring and result history
- Performance analytics and learning progress
- Course completion certificates
- Learning resource browsing and downloading
- Notifications, announcements, achievements, and new learning content
- Skills and interests profile
- Competency-based course and trainer recommendations

### Trainer

- Create and manage courses
- Add and delete course lectures
- Define course-required skills
- Create multi-question assessments
- Delete owned assessments
- Upload course resources
- Monitor enrollment and course completion
- View trainee-level progress
- View assessment attempts and results
- Review average assessment performance

### Administrator

- View platform statistics
- Manage user roles
- Publish announcements, notifications, achievements, and homepage content
- Target publications to specific users
- Edit and delete published content
- Manage published homepage learning content

## 4. Technology Stack

### Frontend

- HTML
- CSS
- Vanilla JavaScript
- GitHub Pages static deployment

### Backend

- Rust
- Axum
- Tokio
- Serde / JSON
- JWT authentication
- SQLite
- Rusqlite

### Testing

- Rust integration-style API tests
- `cargo check`
- `cargo test`
- JavaScript syntax validation with `node --check`

## 5. System Architecture

```text
Trainee / Trainer / Admin
            |
            v
     Static Web Frontend
       (HTML/CSS/JS)
            |
            v
       Rust Axum API
            |
            v
          SQLite
```

The frontend is hosted separately from the backend. The frontend communicates with the
backend through authenticated JSON API requests.

## 6. Repository Structure

```text
capacity_connect/
├── README.md
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── db.rs
│       ├── routes/
│       │   ├── auth.rs
│       │   ├── courses.rs
│       │   ├── assessments.rs
│       │   ├── resources.rs
│       │   ├── competencies.rs
│       │   ├── publishing.rs
│       │   ├── admin.rs
│       │   └── users.rs
│       └── tests.rs
└── frontend/
    ├── *.html
    ├── css/
    ├── js/
    └── assets/
```

## 7. Local Setup

### Requirements

- Rust and Cargo
- Node.js (optional, for local static serving and frontend checks)
- A modern browser

### Start the backend

```powershell
Set-Location backend
cargo run
```

The API starts at:

```text
http://localhost:6969
```

### Start the frontend

Serve the `frontend` directory with any static file server. For example, using Node.js:

```powershell
npx serve frontend
```

Then open:

```text
http://localhost:5500
```

Opening HTML files directly with `file://` may cause browser API or CORS issues.

## 8. Demo Administrator Account

For the demo deployment, the backend seeds an administrator account on first startup:

```text
Email: admin@gmail.com
Password: 123456789
```

The values can be changed with environment variables:

```text
ADMIN_EMAIL=admin@gmail.com
ADMIN_PASSWORD=123456789
```

This account is intended for the resettable demo environment and should be changed for
any persistent or public production deployment.

## 9. Deployment

The intended deployment arrangement is:

- **Frontend:** GitHub Pages
- **Backend:** Render
- **Database:** SQLite

### Backend environment variables

```text
HOST=0.0.0.0
PORT=<host-provided-port>
JWT_SECRET=<long-random-secret>
ADMIN_EMAIL=admin@gmail.com
ADMIN_PASSWORD=123456789
```

The JWT secret must be configured on the backend host and must not be placed in the
frontend repository.

### Frontend backend URL

Set the backend origin in:

- `frontend/js/auth.js`
- `frontend/js/api.js`
- `frontend/js/home.js`

Use the backend origin without `/api`, for example:

```javascript
window.CAPACITY_API_BASE = "https://your-backend.example.com";
```

The frontend appends `/api` automatically.

## 10. Testing

Run backend checks from the `backend` directory:

```powershell
cargo check
cargo test
```

The automated backend tests cover:

- Authentication and role access
- Assessment creation, attempts, and results
- Course enrollment and lecture progress
- Resource upload, download, and ownership rules

## 11. Future Scope

- Persistent production database storage
- Richer trainer recommendation ranking
- Advanced reporting and downloadable analytics
- More granular notification preferences
