use circa_backend::{
    event::{
        entity, event_branding_entity, membership_entity,
        models::{
            Event, EventBranding, EventCollaborator, EventMembershipRole, EventStatus, PlannerItem,
            PlannerTimelineItem, SocialPost,
        },
        planner_item_entity, planner_timeline_item_entity, social_post_entity,
    },
    user::entity as user_entity,
};

#[test]
fn event_status_converts_between_api_and_entity_enums() {
    for (api, entity_value, label) in [
        (EventStatus::Draft, entity::Status::Draft, "Draft"),
        (EventStatus::Active, entity::Status::Active, "Active"),
        (EventStatus::Closed, entity::Status::Closed, "Closed"),
        (EventStatus::Archived, entity::Status::Archived, "Archived"),
        (
            EventStatus::PendingDestruction,
            entity::Status::PendingDestruction,
            "Pending destruction",
        ),
    ] {
        assert_eq!(entity::Status::from(api.clone()), entity_value);
        assert_eq!(EventStatus::from(entity_value), api);
        assert_eq!(api.to_string(), label);
    }
}

#[test]
fn event_membership_role_converts_between_api_and_entity_enums() {
    for (api, entity_value, label) in [
        (
            EventMembershipRole::Owner,
            membership_entity::Role::Owner,
            "Owner",
        ),
        (
            EventMembershipRole::Organizer,
            membership_entity::Role::Organizer,
            "Organizer",
        ),
        (
            EventMembershipRole::Staff,
            membership_entity::Role::Staff,
            "Staff",
        ),
        (
            EventMembershipRole::Volunteer,
            membership_entity::Role::Volunteer,
            "Volunteer",
        ),
    ] {
        assert_eq!(membership_entity::Role::from(api.clone()), entity_value);
        assert_eq!(EventMembershipRole::from(entity_value), api);
        assert_eq!(api.to_string(), label);
    }
}

#[test]
fn event_from_parts_maps_entity_model_and_current_user_role() {
    let event = Event::from_parts(
        entity::Model {
            id: "event-1".to_string(),
            name: "Event".to_string(),
            slug: "event".to_string(),
            description: "Desc".to_string(),
            venue: "Venue".to_string(),
            timezone: "UTC".to_string(),
            starts_at: "2026-01-01T00:00:00Z".to_string(),
            ends_at: "2026-01-02T00:00:00Z".to_string(),
            status: entity::Status::Active,
            created_by_user_id: "user-1".to_string(),
            destruction_requested_at: Some("2026-01-03T00:00:00Z".to_string()),
            created_at: "created".to_string(),
            updated_at: "updated".to_string(),
        },
        membership_entity::Role::Organizer,
    );

    assert_eq!(event.id, "event-1");
    assert_eq!(event.status, EventStatus::Active);
    assert_eq!(event.current_user_role, EventMembershipRole::Organizer);
    assert_eq!(
        event.destruction_requested_at,
        Some("2026-01-03T00:00:00Z".to_string())
    );
}

#[test]
fn event_collaborator_from_parts_combines_membership_and_user() {
    let collaborator = EventCollaborator::from_parts(
        membership_entity::Model {
            id: "membership-1".to_string(),
            event_id: "event-1".to_string(),
            user_id: "user-1".to_string(),
            role: membership_entity::Role::Staff,
            created_at: "created".to_string(),
        },
        user_entity::Model {
            id: "user-1".to_string(),
            name: "Ada".to_string(),
            surname: "Lovelace".to_string(),
            email: "ada@example.com".to_string(),
            phone: "+48".to_string(),
            role: user_entity::Role::Staff,
            status: user_entity::Status::Active,
            availability_hours: "[]".to_string(),
        },
    );

    assert_eq!(collaborator.user_id, "user-1");
    assert_eq!(collaborator.role, EventMembershipRole::Staff);
    assert_eq!(collaborator.email, "ada@example.com");
}

#[test]
fn event_branding_maps_model_and_default_state() {
    let default = EventBranding::default_for_event("event-1");
    assert_eq!(default.id, None);
    assert_eq!(default.event_id, "event-1");
    assert_eq!(default.theme_mode, "dark");

    let mapped = EventBranding::from_model(event_branding_entity::Model {
        id: "branding-1".to_string(),
        event_id: "event-1".to_string(),
        event_name_override: "Override".to_string(),
        tagline: "Tag".to_string(),
        primary_color: "#111111".to_string(),
        secondary_color: "#222222".to_string(),
        theme_mode: "light".to_string(),
        background_color: "#ffffff".to_string(),
        notes: "Notes".to_string(),
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
    });

    assert_eq!(mapped.id, Some("branding-1".to_string()));
    assert_eq!(mapped.theme_mode, "light");
    assert_eq!(mapped.created_at, Some("created".to_string()));
}

#[test]
fn planner_timeline_and_social_models_convert_to_api_records() {
    let planner: PlannerItem = planner_item_entity::Model {
        id: "planner-1".to_string(),
        event_id: "event-1".to_string(),
        title: "Task".to_string(),
        notes: "Notes".to_string(),
        position: 1,
        done: true,
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
    }
    .into();
    assert_eq!(planner.title, "Task");
    assert!(planner.done);

    let timeline: PlannerTimelineItem = planner_timeline_item_entity::Model {
        id: "timeline-1".to_string(),
        event_id: "event-1".to_string(),
        title: "Timeline".to_string(),
        item_type: "task".to_string(),
        starts_at: "start".to_string(),
        ends_at: "end".to_string(),
        status: "planned".to_string(),
        owner: "Ada".to_string(),
        notes: "Notes".to_string(),
        color: "#abcdef".to_string(),
        position: 2,
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
        depends_on_item_id: "planner-1".to_string(),
        assigned_user_id: "user-1".to_string(),
    }
    .into();
    assert_eq!(timeline.depends_on_item_id, "planner-1");
    assert_eq!(timeline.assigned_user_id, "user-1");

    let social: SocialPost = social_post_entity::Model {
        id: "post-1".to_string(),
        event_id: "event-1".to_string(),
        platform: "Mastodon".to_string(),
        title: "Post".to_string(),
        body: "Body".to_string(),
        status: "ready".to_string(),
        position: 3,
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
    }
    .into();
    assert_eq!(social.status, "ready");
}
