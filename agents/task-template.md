## Task: Add user authentication

### Scope
- Implement JWT-based login
- Add password hashing with bcrypt
- Create login/logout endpoints

### Acceptance Criteria
- [ ] User can register with email/password
- [ ] User can login and receive JWT token
- [ ] Protected routes reject invalid tokens
- [ ] Passwords are hashed, not stored plain text
- [ ] Unit tests cover happy path and error cases

### Known Patterns
- Use existing database connection pool
- Follow project's error handling conventions
- Use Zod for input validation

### Scope Limits
- Do NOT modify existing user schema
- Do NOT add OAuth providers (out of scope)
- Escalate to human if database migration needed