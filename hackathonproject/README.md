# The Thing — Training Portal Frontend

A multi-page HTML/CSS/JavaScript prototype based on the requested Training Portal UI.

## Included
- Bird SVG logo and renamed brand: **The Thing**
- User-entered name displayed across dashboard/profile/certificate/result pages
- No dashboard/progress card on login or signup pages
- Login/signup gate for protected pages
- Role-specific Trainee, Trainer and Admin dashboards
- Separate Notifications page
- Settings page with dark/light mode, browser-notification and compact-layout settings
- Assessment instructions + complete MCQ option rendering
- Timed assessment with answers saved in localStorage
- Assessment result page after submission
- Certificate locked until course progress is at least 80%
- Course page demo button to increase progress by 10% for testing the certificate rule
- Responsive layout

## Run
Open `index.html` in VS Code, preferably with Live Server.

## Demo authentication
This is a frontend prototype, so any valid-looking name/email/password is accepted. The selected role controls which dashboard opens. Real authentication and authorization must be enforced by the backend in production; localStorage is not secure authentication.

## Authentication architecture fix (v5)
The previous loop was caused by having authentication logic split across two scripts and redirects being performed from DOMContentLoaded. The new version has one authentication authority (`window.ThingAuth`) in `js/auth.js`, performs protected-page checks immediately, never uses a `next` URL, and sends a successful login/signup directly to the selected role dashboard. `app.js` no longer performs authentication redirects.
