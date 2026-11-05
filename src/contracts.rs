//! Canonical generated protobuf types shared by every Benito backend service.

pub mod analytics {
    tonic::include_proto!("analytics");
}
pub mod auth {
    tonic::include_proto!("auth");
}
pub mod chat {
    tonic::include_proto!("chat");
}
pub mod device {
    tonic::include_proto!("device");
}
pub mod media {
    tonic::include_proto!("media");
}
pub mod notification {
    tonic::include_proto!("notification");
}
pub mod role {
    tonic::include_proto!("role");
}
pub mod users {
    tonic::include_proto!("users");
}

pub mod benito {
    pub mod discovery {
        pub mod internal {
            pub mod v1 {
                tonic::include_proto!("benito.discovery.internal.v1");
            }
        }
    }

    pub mod media {
        pub mod internal {
            pub mod v1 {
                tonic::include_proto!("benito.media.internal.v1");
            }
        }
    }
    pub mod public {
        #[allow(clippy::enum_variant_names, reason = "canonical protobuf names")]
        pub mod v1 {
            tonic::include_proto!("benito.public.v1");
        }
    }
    pub mod realtime {
        pub mod internal {
            pub mod v1 {
                tonic::include_proto!("benito.realtime.internal.v1");
            }
        }
        pub mod v1 {
            tonic::include_proto!("benito.realtime.v1");
        }
    }
}

pub use benito::discovery::internal::v1 as discovery_internal_v1;
pub use benito::media::internal::v1 as media_internal_v1;
pub use benito::public::v1 as public_v1;
pub use benito::realtime::internal::v1 as realtime_internal_v1;
pub use benito::realtime::v1 as realtime_v1;
