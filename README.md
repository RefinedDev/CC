# The Thing — Training Portal Frontend

This version adds functional demo course enrollment and course-specific learning flows.

## New features
- Click **Enroll** on Course Catalog → opens `enrollment.html?course=...`.
- Enrollment form collects name, email and password confirmation details.
- Enrollment is stored in `localStorage` under `thing_enrollments`.
- After enrollment, the course card shows **Start Learning**.
- Start Learning opens the matching `lectures.html?course=...` page.
- Starting a lecture changes the course action to **Continue Learning**.
- Course progress is NOT increased by clicking Start Learning or opening a lecture.
- Completing a lecture increases progress based on the course's total lecture count.
- Feedback now has a selectable 1–5 star rating.
- Existing light/dark theme, role authentication and shared UI remain intact.

## Demo note
The enrollment password is only validated on the frontend and is intentionally not stored. For production, enrollment/login verification should be implemented on the backend.
