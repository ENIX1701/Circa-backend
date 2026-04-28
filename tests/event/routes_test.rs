use crate::common::{
    seed_basic_event, seed_branding, seed_event, seed_membership, seed_planner_item,
    seed_social_post, seed_timeline_item, seed_user, setup_db,
};
use circa_backend::{
    event::{
        models::{
            AddEventCollaboratorRequest, CreateEventRequest, CreatePlannerItemRequest,
            CreatePlannerTimelineItemRequest, CreateSocialMediaPostRequest, EventMembershipRole,
            EventStatus, UpdateEventCollaboratorRequest, UpdatePlannerItemRequest,
            UpdatePlannerTimelineItemRequest, UpdateSocialMediaPostRequest,
            UpsertEventBrandingRequest,
        },
        repository::EventRepository,
        service::EventService,
    },
    user::entity::Entity as UserEntity,
};
use sea_orm::EntityTrait;

fn valid_event_request(slug: &str) -> CreateEventRequest {
    CreateEventRequest {
        name: "Launch Party".to_string(),
        slug: slug.to_string(),
        description: Some("Description".to_string()),
        venue: "Expo".to_string(),
        timezone: "Europe/Warsaw".to_string(),
        starts_at: "2026-05-01T10:00:00Z".to_string(),
        ends_at: "2026-05-02T10:00:00Z".to_string(),
    }
}

fn service(db: &sea_orm::DatabaseConnection) -> EventService {
    EventService::new(EventRepository::new(db.clone()))
}

#[actix_web::test]
async fn create_event_validates_input_and_creates_owner_membership() {
    let db = setup_db().await;
    seed_user(&db, "owner-1", "owner@circa.local", "admin", "active").await;
    let service = service(&db);

    let event = service
        .create_event(valid_event_request("launch-party"), "owner-1")
        .await
        .unwrap();

    assert_eq!(event.status, EventStatus::Draft);
    assert_eq!(event.current_user_role, EventMembershipRole::Owner);
    assert_eq!(event.created_by_user_id, "owner-1");

    for (req, message) in [
        (
            {
                let mut req = valid_event_request("bad-name");
                req.name = " ".to_string();
                req
            },
            "Bad request: Event name is required",
        ),
        (
            {
                let mut req = valid_event_request("");
                req.slug = " ".to_string();
                req
            },
            "Bad request: Event slug is required",
        ),
        (
            { valid_event_request("Bad Slug") },
            "Bad request: Event slug must contain only lowercase letters, numbers and huphens",
        ),
        (
            {
                let mut req = valid_event_request("bad-venue");
                req.venue = " ".to_string();
                req
            },
            "Bad request: Venue is required",
        ),
        (
            {
                let mut req = valid_event_request("bad-timezone");
                req.timezone = " ".to_string();
                req
            },
            "Bad request: Timezone is required",
        ),
        (
            {
                let mut req = valid_event_request("bad-start");
                req.starts_at = "bad".to_string();
                req
            },
            "Bad request: starts_at must be a valid RFC3339 datetime",
        ),
        (
            {
                let mut req = valid_event_request("bad-end");
                req.ends_at = "bad".to_string();
                req
            },
            "Bad request: ends_at must be a valid RFC3339 datetime",
        ),
        (
            {
                let mut req = valid_event_request("bad-order");
                req.starts_at = "2026-05-02T10:00:00Z".to_string();
                req.ends_at = "2026-05-01T10:00:00Z".to_string();
                req
            },
            "Bad request: ends_at must be later than starts_at",
        ),
    ] {
        let err = service.create_event(req, "owner-1").await.unwrap_err();
        assert_eq!(err.to_string(), message);
    }
}

#[actix_web::test]
async fn event_access_and_listing_are_membership_scoped() {
    let db = setup_db().await;
    seed_user(&db, "owner-1", "owner@circa.local", "admin", "active").await;
    seed_user(
        &db,
        "outsider-1",
        "outsider@circa.local",
        "volunteer",
        "active",
    )
    .await;
    seed_event(&db, "event-1", "draft", "owner-1").await;
    seed_event(&db, "event-2", "active", "owner-1").await;
    seed_membership(&db, "event-1", "owner-1", "owner").await;

    let service = service(&db);

    assert_eq!(
        service.get_events_for_user("owner-1").await.unwrap().len(),
        1
    );
    assert!(
        service
            .get_event_for_user("event-1", "owner-1")
            .await
            .is_ok()
    );
    assert_eq!(
        service
            .get_event_for_user("event-1", "outsider-1")
            .await
            .unwrap_err()
            .to_string(),
        "Not found: Event not found"
    );
}

#[actix_web::test]
async fn lifecycle_transitions_enforce_owner_and_status_rules() {
    let db = setup_db().await;
    seed_basic_event(&db, "event-1", "draft").await;
    let service = service(&db);

    assert_eq!(
        service
            .activate_event("event-1", "organizer-1")
            .await
            .unwrap_err()
            .to_string(),
        "Forbidden"
    );

    assert_eq!(
        service
            .activate_event("missing", "owner-1")
            .await
            .unwrap_err()
            .to_string(),
        "Not found: Event not found"
    );

    assert_eq!(
        service
            .activate_event("event-1", "owner-1")
            .await
            .unwrap()
            .status,
        EventStatus::Active
    );

    assert_eq!(
        service
            .activate_event("event-1", "owner-1")
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: Only draft events can be activated"
    );

    assert_eq!(
        service
            .close_event("event-1", "owner-1")
            .await
            .unwrap()
            .status,
        EventStatus::Closed
    );

    assert_eq!(
        service
            .request_destruction("event-1", "owner-1")
            .await
            .unwrap()
            .status,
        EventStatus::PendingDestruction
    );

    assert_eq!(
        service
            .cancel_destruction("event-1", "owner-1")
            .await
            .unwrap()
            .status,
        EventStatus::Closed
    );

    assert_eq!(
        service
            .archive_event("event-1", "owner-1")
            .await
            .unwrap()
            .status,
        EventStatus::Archived
    );
}

#[actix_web::test]
async fn collaborators_can_be_listed_added_updated_and_removed_by_owner() {
    let db = setup_db().await;
    seed_basic_event(&db, "event-1", "active").await;
    seed_user(&db, "new-user", "new@circa.local", "volunteer", "active").await;
    let service = service(&db);

    assert_eq!(
        service
            .get_event_collaborators_for_user("event-1", "staff-1")
            .await
            .unwrap()
            .len(),
        4
    );

    let added = service
        .add_event_collaborator(
            "event-1",
            "owner-1",
            AddEventCollaboratorRequest {
                email: "NEW@CIRCA.LOCAL".to_string(),
                role: EventMembershipRole::Volunteer,
            },
        )
        .await
        .unwrap();
    assert_eq!(added.user_id, "new-user");

    let updated = service
        .update_event_collaborator(
            "event-1",
            "new-user",
            "owner-1",
            UpdateEventCollaboratorRequest {
                role: EventMembershipRole::Staff,
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.role, EventMembershipRole::Staff);

    service
        .delete_event_collaborator("event-1", "new-user", "owner-1")
        .await
        .unwrap();

    assert_eq!(
        service
            .add_event_collaborator(
                "event-1",
                "organizer-1",
                AddEventCollaboratorRequest {
                    email: "new@circa.local".to_string(),
                    role: EventMembershipRole::Volunteer,
                },
            )
            .await
            .unwrap_err()
            .to_string(),
        "Forbidden"
    );

    assert_eq!(
        service
            .add_event_collaborator(
                "event-1",
                "owner-1",
                AddEventCollaboratorRequest {
                    email: " ".to_string(),
                    role: EventMembershipRole::Volunteer,
                },
            )
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: Email is required :c"
    );

    assert_eq!(
        service
            .update_event_collaborator(
                "event-1",
                "owner-1",
                "owner-1",
                UpdateEventCollaboratorRequest {
                    role: EventMembershipRole::Staff,
                },
            )
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: You cannot change your own event role :c"
    );

    assert_eq!(
        service
            .delete_event_collaborator("event-1", "owner-1", "owner-1")
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: You cannot remove yourself from the event >:C"
    );
}

#[actix_web::test]
async fn branding_supports_default_get_upsert_update_and_validation() {
    let db = setup_db().await;
    seed_basic_event(&db, "event-1", "active").await;
    let service = service(&db);

    let default = service
        .get_event_branding_for_user("event-1", "volunteer-1")
        .await
        .unwrap();
    assert_eq!(default.id, None);
    assert_eq!(default.theme_mode, "dark");

    let saved = service
        .upsert_event_branding(
            "event-1",
            "organizer-1",
            UpsertEventBrandingRequest {
                event_name_override: "Override".to_string(),
                tagline: "Tag".to_string(),
                primary_color: "#123456".to_string(),
                secondary_color: "#abcdef".to_string(),
                theme_mode: "light".to_string(),
                background_color: "#ffffff".to_string(),
                notes: "Notes".to_string(),
            },
        )
        .await
        .unwrap();
    assert_eq!(saved.theme_mode, "light");

    let updated = service
        .upsert_event_branding(
            "event-1",
            "owner-1",
            UpsertEventBrandingRequest {
                event_name_override: "Updated".to_string(),
                tagline: "".to_string(),
                primary_color: "".to_string(),
                secondary_color: "".to_string(),
                theme_mode: "".to_string(),
                background_color: "".to_string(),
                notes: "".to_string(),
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.event_name_override, "Updated");
    assert_eq!(updated.theme_mode, "dark");

    for req in [
        UpsertEventBrandingRequest {
            primary_color: "bad".to_string(),
            ..valid_branding_request()
        },
        UpsertEventBrandingRequest {
            secondary_color: "bad".to_string(),
            ..valid_branding_request()
        },
        UpsertEventBrandingRequest {
            background_color: "bad".to_string(),
            ..valid_branding_request()
        },
        UpsertEventBrandingRequest {
            theme_mode: "sepia".to_string(),
            ..valid_branding_request()
        },
    ] {
        assert!(
            service
                .upsert_event_branding("event-1", "owner-1", req)
                .await
                .is_err()
        );
    }

    assert_eq!(
        service
            .upsert_event_branding("event-1", "volunteer-1", valid_branding_request())
            .await
            .unwrap_err()
            .to_string(),
        "Forbidden"
    );
}

fn valid_branding_request() -> UpsertEventBrandingRequest {
    UpsertEventBrandingRequest {
        event_name_override: "".to_string(),
        tagline: "".to_string(),
        primary_color: "#111111".to_string(),
        secondary_color: "#222222".to_string(),
        theme_mode: "dark".to_string(),
        background_color: "#333333".to_string(),
        notes: "".to_string(),
    }
}

#[actix_web::test]
async fn planner_items_are_content_manager_scoped_and_validated() {
    let db = setup_db().await;
    seed_basic_event(&db, "event-1", "active").await;
    seed_planner_item(&db, "event-1", "planner-1", 0).await;
    let service = service(&db);

    assert_eq!(
        service
            .get_planner_items_for_user("event-1", "volunteer-1")
            .await
            .unwrap()
            .len(),
        1
    );

    let created = service
        .create_planner_item(
            "event-1",
            "staff-1",
            CreatePlannerItemRequest {
                title: "New".to_string(),
                notes: Some("Notes".to_string()),
            },
        )
        .await
        .unwrap();
    assert_eq!(created.position, 1);

    assert_eq!(
        service
            .create_planner_item(
                "event-1",
                "staff-1",
                CreatePlannerItemRequest {
                    title: " ".to_string(),
                    notes: None,
                },
            )
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: Planner item title is required"
    );

    assert_eq!(
        service
            .create_planner_item(
                "event-1",
                "volunteer-1",
                CreatePlannerItemRequest {
                    title: "Nope".to_string(),
                    notes: None,
                },
            )
            .await
            .unwrap_err()
            .to_string(),
        "Forbidden"
    );

    let updated = service
        .update_planner_item(
            "event-1",
            &created.id,
            "organizer-1",
            UpdatePlannerItemRequest {
                title: Some("Updated".to_string()),
                notes: Some("Updated notes".to_string()),
                position: Some(5),
                done: Some(true),
            },
        )
        .await
        .unwrap();
    assert!(updated.done);
    assert_eq!(updated.position, 5);

    for req in [
        UpdatePlannerItemRequest {
            title: None,
            notes: None,
            position: None,
            done: None,
        },
        UpdatePlannerItemRequest {
            title: Some(" ".to_string()),
            notes: None,
            position: None,
            done: None,
        },
        UpdatePlannerItemRequest {
            title: None,
            notes: None,
            position: Some(-1),
            done: None,
        },
    ] {
        assert!(
            service
                .update_planner_item("event-1", &created.id, "owner-1", req)
                .await
                .is_err()
        );
    }

    service
        .delete_planner_item("event-1", &created.id, "owner-1")
        .await
        .unwrap();
}

#[actix_web::test]
async fn timeline_items_are_content_manager_scoped_and_validated() {
    let db = setup_db().await;
    seed_basic_event(&db, "event-1", "active").await;
    seed_timeline_item(&db, "event-1", "timeline-1", 0).await;
    let service = service(&db);

    assert_eq!(
        service
            .get_planner_timeline_items_for_user("event-1", "volunteer-1")
            .await
            .unwrap()
            .len(),
        1
    );

    let created = service
        .create_planner_timeline_item("event-1", "staff-1", valid_timeline_create("New timeline"))
        .await
        .unwrap();
    assert_eq!(created.position, 1);

    for req in [
        CreatePlannerTimelineItemRequest {
            title: " ".to_string(),
            ..valid_timeline_create("unused")
        },
        CreatePlannerTimelineItemRequest {
            item_type: "invalid".to_string(),
            ..valid_timeline_create("unused")
        },
        CreatePlannerTimelineItemRequest {
            status: Some("invalid".to_string()),
            ..valid_timeline_create("unused")
        },
        CreatePlannerTimelineItemRequest {
            color: Some("bad".to_string()),
            ..valid_timeline_create("unused")
        },
        CreatePlannerTimelineItemRequest {
            starts_at: "bad".to_string(),
            ..valid_timeline_create("unused")
        },
        CreatePlannerTimelineItemRequest {
            starts_at: "2026-05-02T10:00:00Z".to_string(),
            ends_at: "2026-05-01T10:00:00Z".to_string(),
            ..valid_timeline_create("unused")
        },
        CreatePlannerTimelineItemRequest {
            assigned_user_id: Some("missing-user".to_string()),
            ..valid_timeline_create("unused")
        },
    ] {
        assert!(
            service
                .create_planner_timeline_item("event-1", "owner-1", req)
                .await
                .is_err()
        );
    }

    let updated = service
        .update_planner_timeline_item(
            "event-1",
            &created.id,
            "owner-1",
            UpdatePlannerTimelineItemRequest {
                title: Some("Updated".to_string()),
                item_type: Some("milestone".to_string()),
                starts_at: Some("2026-05-01T11:00:00Z".to_string()),
                ends_at: Some("2026-05-01T12:00:00Z".to_string()),
                status: Some("done".to_string()),
                owner: Some("Owner".to_string()),
                notes: Some("Notes".to_string()),
                color: Some("#123456".to_string()),
                depends_on_item_id: Some("timeline-1".to_string()),
                assigned_user_id: Some("staff-1".to_string()),
                position: Some(10),
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.status, "done");
    assert_eq!(updated.position, 10);

    assert_eq!(
        service
            .update_planner_timeline_item(
                "event-1",
                &created.id,
                "owner-1",
                UpdatePlannerTimelineItemRequest {
                    title: None,
                    item_type: None,
                    starts_at: None,
                    ends_at: None,
                    status: None,
                    owner: None,
                    notes: None,
                    color: None,
                    depends_on_item_id: None,
                    assigned_user_id: None,
                    position: None,
                },
            )
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: At least one timeline field must be provided"
    );

    service
        .delete_planner_timeline_item("event-1", &created.id, "owner-1")
        .await
        .unwrap();
}

fn valid_timeline_create(title: &str) -> CreatePlannerTimelineItemRequest {
    CreatePlannerTimelineItemRequest {
        title: title.to_string(),
        item_type: "task".to_string(),
        starts_at: "2026-05-01T10:00:00Z".to_string(),
        ends_at: "2026-05-01T12:00:00Z".to_string(),
        status: Some("planned".to_string()),
        owner: Some("Owner".to_string()),
        notes: Some("Notes".to_string()),
        color: Some("#abcdef".to_string()),
        depends_on_item_id: None,
        assigned_user_id: Some("staff-1".to_string()),
    }
}

#[actix_web::test]
async fn social_posts_are_content_manager_scoped_and_validated() {
    let db = setup_db().await;
    seed_basic_event(&db, "event-1", "active").await;
    seed_social_post(&db, "event-1", "post-1", 0).await;
    let service = service(&db);

    assert_eq!(
        service
            .get_social_posts_for_user("event-1", "volunteer-1")
            .await
            .unwrap()
            .len(),
        1
    );

    let created = service
        .create_social_post(
            "event-1",
            "staff-1",
            CreateSocialMediaPostRequest {
                platform: "Mastodon".to_string(),
                title: "New post".to_string(),
                body: Some("Body".to_string()),
            },
        )
        .await
        .unwrap();
    assert_eq!(created.status, "draft");
    assert_eq!(created.position, 1);

    for req in [
        CreateSocialMediaPostRequest {
            platform: " ".to_string(),
            title: "Title".to_string(),
            body: None,
        },
        CreateSocialMediaPostRequest {
            platform: "Mastodon".to_string(),
            title: " ".to_string(),
            body: None,
        },
    ] {
        assert!(
            service
                .create_social_post("event-1", "owner-1", req)
                .await
                .is_err()
        );
    }

    let updated = service
        .update_social_post(
            "event-1",
            &created.id,
            "owner-1",
            UpdateSocialMediaPostRequest {
                platform: Some("Bluesky".to_string()),
                title: Some("Updated".to_string()),
                body: Some("Updated body".to_string()),
                status: Some("ready".to_string()),
                position: Some(9),
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.status, "ready");
    assert_eq!(updated.position, 9);

    for req in [
        UpdateSocialMediaPostRequest {
            platform: None,
            title: None,
            body: None,
            status: None,
            position: None,
        },
        UpdateSocialMediaPostRequest {
            platform: Some(" ".to_string()),
            title: None,
            body: None,
            status: None,
            position: None,
        },
        UpdateSocialMediaPostRequest {
            platform: None,
            title: Some(" ".to_string()),
            body: None,
            status: None,
            position: None,
        },
        UpdateSocialMediaPostRequest {
            platform: None,
            title: None,
            body: None,
            status: Some("invalid".to_string()),
            position: None,
        },
        UpdateSocialMediaPostRequest {
            platform: None,
            title: None,
            body: None,
            status: None,
            position: Some(-1),
        },
    ] {
        assert!(
            service
                .update_social_post("event-1", &created.id, "owner-1", req)
                .await
                .is_err()
        );
    }

    service
        .delete_social_post("event-1", &created.id, "owner-1")
        .await
        .unwrap();
}

#[actix_web::test]
async fn export_event_aggregates_all_event_data() {
    let db = setup_db().await;
    seed_basic_event(&db, "event-1", "closed").await;
    seed_branding(&db, "event-1").await;
    seed_planner_item(&db, "event-1", "planner-1", 0).await;
    seed_timeline_item(&db, "event-1", "timeline-1", 0).await;
    seed_social_post(&db, "event-1", "post-1", 0).await;

    let export = service(&db)
        .export_event("event-1", "owner-1")
        .await
        .unwrap();

    assert_eq!(export.event.id, "event-1");
    assert_eq!(export.branding.event_name_override, "Override");
    assert_eq!(export.planner_items.len(), 1);
    assert_eq!(export.planner_timeline_items.len(), 1);
    assert_eq!(export.social_posts.len(), 1);
    assert!(!export.exported_at.is_empty());

    assert_eq!(UserEntity::find().all(&db).await.unwrap().len(), 4);
}
