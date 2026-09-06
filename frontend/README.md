# Capacity Connect — Training Portal Frontend

This version integrates course enrollment, course-specific learning, and progress tracking with the backend API.

## New features

- Click **Enroll** on Course Catalog → opens `enrollment.html?course=...`.
- Enrollment uses the signed-in account and is stored by the backend.
- After enrollment, the course card shows **Continue Learning**.
- Start Learning opens the matching `lectures.html?course=...` page.
- Starting a lecture changes the course action to **Continue Learning**.
- Course progress is stored per user by the backend.
- Completing a lecture updates backend progress based on the course's total lecture count.
- Feedback now has a selectable 1–5 star rating.
- Existing light/dark theme, role authentication and shared UI remain intact.

## Current scope

Assessments are backed by SQLite: trainers can create timed subject-based MCQs with multiple questions, trainees can submit attempts, and scores are persisted and displayed on the result page.

Feedback, library, notifications, and certificate content still use frontend-only demo data until their backend endpoints are implemented.
