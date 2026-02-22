# Trainer Panel

A lightweight web panel for trainer users to issue training commands to JeebsAI.

## Access
- URL: `/webui/trainer_panel.html`
- Requires a logged-in account with role `trainer` (or root admin).

## Trainer Commands
- `train help` — List trainer commands.
- `train: <topic>` — Set the training focus topic.
- `train on` — Enable training mode.
- `train off` — Disable training mode.

## How It Works
- Uses `/api/auth/status` to check `is_trainer` or root admin.
- Sends commands to `/api/chat` for execution.

## Quick Try
1. Assign role `trainer` to a user (admin only).
2. Log in as that user.
3. Open `/webui/trainer_panel.html`.
4. Set focus or toggle training.

## Notes
- The panel only shows actions for trainers or root admin.
- All commands are audited in server logs.
