# Roles

Roles in Circa are a fundamental part of the authorization schema. They act as access guardrails. The higher and more important the role - the bigger permissions it grants.

## Overview

The system currently implements 4 levels of permissions described below.

### Admin

Self explanatory. Technical lead of the project, can do everything `>:3c`

### Organizer

Trusted right-hand people. Full management of people and assets, cannot manage the event's destruction.

### Staff

Scoped responsibilities. Can update statuses, update their own info and profiles, and that's about it... Sorry x3

### Volunteer

Read-only access to (hopefully) scoped dashboards. Coordinated by other roles, they're here to help and have a good time!

## Dashboard access

| Dashboard     | Admin     | Organizer | Staff                 | Volunteer                                                             |
|---------------|-----------|-----------|-----------------------|-----------------------------------------------------------------------|
| **Branding**  | Full edit | Full edit | View only             | No access/view only (need to think about it x3) (maybe configurable?) |
| **Staff**     | Full edit | Full edit | View only             | Own profile only                                                      |
| **Logistics** | Full edit | Full edit | Update status         | No access                                                             |
| **Planner**   | Full edit | Full edit | View + complete tasks | Own schedule only                                                     |
| **Socials**   | Full edit | Full edit | View only             | No access                                                             |
| **Export**    | 🙂‍↕️       | nuh uh    | you wish...           | x3                                                                    |

> [!CAUTION]
> While I'd like to see *all* of this implemented, the event ends in a month, so only some may come to fruition QwQ (at least for now)
