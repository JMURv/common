//! Shared Testcontainers harness for service-local and cross-service integration scenarios.

use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, ContainerRequest, GenericImage, ImageExt, TestcontainersError};
use thiserror::Error;

const SUITE_LABEL: &str = "backend";

pub struct TestService {
    pub host: String,
    pub port: u16,
    _container: ContainerAsync<GenericImage>,
}

#[derive(Debug, Error)]
pub enum TestServiceError {
    #[error("failed to start the {service} test container")]
    Start {
        service: &'static str,
        #[source]
        source: TestcontainersError,
    },
    #[error("failed to resolve the host of the {service} test container")]
    Host {
        service: &'static str,
        #[source]
        source: TestcontainersError,
    },
    #[error("failed to resolve host port for {service} test container port {internal_port}")]
    Port {
        service: &'static str,
        internal_port: u16,
        #[source]
        source: TestcontainersError,
    },
}

impl TestService {
    /// Останавливает только контейнер этой fixture для проверки отказа зависимости.
    pub async fn stop(&self) -> Result<(), TestcontainersError> {
        self._container.stop().await
    }

    pub fn http_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    pub fn tcp_url(&self, scheme: &str) -> String {
        format!("{scheme}://{}:{}", self.host, self.port)
    }
}

#[track_caller]
fn require_service(result: Result<TestService, TestServiceError>) -> TestService {
    result.unwrap_or_else(|error| {
        let mut message = error.to_string();
        let mut source = std::error::Error::source(&error);
        while let Some(cause) = source {
            message.push_str("\ncaused by: ");
            message.push_str(&cause.to_string());
            source = cause.source();
        }
        panic!("test service setup failed:\n{message}");
    })
}

/// Запускает собственную fixture и возвращает endpoint после её условий готовности.
pub async fn start(
    service: &'static str,
    image: ContainerRequest<GenericImage>,
    internal_port: u16,
) -> Result<TestService, TestServiceError> {
    let container = image
        .with_label("benito.test-suite", SUITE_LABEL)
        .start()
        .await
        .map_err(|source| TestServiceError::Start { service, source })?;
    let host = container.get_host().await.map_err(|source| TestServiceError::Host { service, source })?;
    let port = container.get_host_port_ipv4(internal_port).await.map_err(|source| TestServiceError::Port {
        service,
        internal_port,
        source,
    })?;
    Ok(TestService { host: host.to_string(), port, _container: container })
}

pub async fn postgres() -> TestService {
    require_service(try_postgres().await)
}

pub async fn try_postgres() -> Result<TestService, TestServiceError> {
    start("PostgreSQL", postgres_image(), 5432).await
}

/// PostgreSQL с ожиданием init scripts и окончательного TCP server; допускает настройку fixture.
pub fn postgres_image() -> ContainerRequest<GenericImage> {
    GenericImage::new("postgres", "18.1-alpine")
        .with_exposed_port(ContainerPort::Tcp(5432))
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .with_ready_conditions(vec![
            // The init server has no TCP listener. Its ready log is not sufficient.
            WaitFor::message_on_stdout("PostgreSQL init process complete; ready for start up."),
            WaitFor::message_on_stderr("database system is ready to accept connections"),
        ])
}

pub async fn redis() -> TestService {
    require_service(try_redis().await)
}

pub async fn try_redis() -> Result<TestService, TestServiceError> {
    start(
        "Redis",
        GenericImage::new("redis", "8.0.3-alpine").with_exposed_port(ContainerPort::Tcp(6379)).with_ready_conditions(
            vec![WaitFor::message_on_stdout("Ready to accept connections"), WaitFor::seconds(1)],
        ),
        6379,
    )
    .await
}

pub async fn nats() -> TestService {
    require_service(try_nats().await)
}

pub async fn try_nats() -> Result<TestService, TestServiceError> {
    start(
        "NATS",
        GenericImage::new("nats", "2.10.14-alpine")
            .with_exposed_port(ContainerPort::Tcp(4222))
            .with_cmd(["-js"])
            .with_ready_conditions(vec![WaitFor::message_on_either_std("Server is ready"), WaitFor::seconds(1)]),
        4222,
    )
    .await
}

pub async fn nats_with_credentials(user: &str, password: &str) -> TestService {
    require_service(try_nats_with_credentials(user, password).await)
}

pub async fn try_nats_with_credentials(user: &str, password: &str) -> Result<TestService, TestServiceError> {
    start(
        "NATS",
        GenericImage::new("nats", "2.10.14-alpine")
            .with_exposed_port(ContainerPort::Tcp(4222))
            .with_cmd(["-js", "--user", user, "--pass", password])
            .with_ready_conditions(vec![WaitFor::message_on_stderr("Server is ready")]),
        4222,
    )
    .await
}

/// SMTP capture для проверки фактической отправки; HTTP API используется только тестами.
/// Возвращает SMTP endpoint и URL API одноразового Mailpit.
pub async fn smtp_capture() -> (TestService, String) {
    let mut service = require_service(
        start(
            "SMTP",
            GenericImage::new(
                "axllent/mailpit",
                "v1.33.0@sha256:9b3a463be15354c0f0eba91697d80e3e0d8a5623ef0e627cacc836bc17fd5231",
            )
            .with_exposed_port(ContainerPort::Tcp(1025))
            .with_exposed_port(ContainerPort::Tcp(8025))
            // В изолированной сети rDNS Docker gateway может задерживать SMTP greeting.
            .with_env_var("MP_SMTP_DISABLE_RDNS", "true")
            .with_ready_conditions(vec![
                WaitFor::message_on_either_std("[smtpd] starting on"),
                WaitFor::http(
                    testcontainers::core::wait::HttpWaitStrategy::new("/api/v1/messages")
                        .with_port(ContainerPort::Tcp(8025))
                        .with_expected_status_code(200_u16),
                ),
            ])
            .with_startup_timeout(std::time::Duration::from_secs(20)),
            1025,
        )
        .await,
    );
    // Порты получены через get_host_port_ipv4; localhost также может указывать на ::1.
    if service.host == "localhost" {
        service.host = "127.0.0.1".into();
    }
    let api_port = service._container.get_host_port_ipv4(8025).await.expect("SMTP fixture API port");
    let api_url = format!("http://{}:{api_port}", service.host);
    (service, api_url)
}

pub async fn rustfs() -> TestService {
    require_service(try_rustfs().await)
}

pub async fn try_rustfs() -> Result<TestService, TestServiceError> {
    let image = std::env::var("BENITO_TEST_RUSTFS_IMAGE").unwrap_or_else(|_| "rustfs/rustfs".into());
    let tag = std::env::var("BENITO_TEST_RUSTFS_TAG")
        .unwrap_or_else(|_| "1.0.4@sha256:ed9210d6411d77ffceb70d94b964a3cb7d225983368bc25bebe01e995293dcc2".into());
    start(
        "RustFS",
        GenericImage::new(image, tag)
            .with_exposed_port(ContainerPort::Tcp(9000))
            .with_env_var("RUSTFS_ACCESS_KEY", "rustfs-test")
            .with_env_var("RUSTFS_SECRET_KEY", "rustfs-test-password")
            .with_env_var("RUSTFS_CONSOLE_ENABLE", "false")
            // Bound each fixture independently while test functions remain parallel.
            .with_host_config_modifier(|config| {
                config.memory = Some(1024 * 1024 * 1024);
                config.nano_cpus = Some(2_000_000_000);
            })
            .with_env_var("RUSTFS_UNSAFE_BYPASS_DISK_CHECK", "true")
            .with_cmd(["/data"])
            .with_ready_conditions(vec![WaitFor::http(
                testcontainers::core::wait::HttpWaitStrategy::new("/health/ready")
                    .with_port(ContainerPort::Tcp(9000))
                    .with_expected_status_code(200_u16),
            )]),
        9000,
    )
    .await
}

pub async fn elasticsearch() -> TestService {
    require_service(try_elasticsearch().await)
}

pub async fn try_elasticsearch() -> Result<TestService, TestServiceError> {
    start(
        "Elasticsearch",
        GenericImage::new("elasticsearch", "9.1.0")
            .with_exposed_port(ContainerPort::Tcp(9200))
            .with_env_var("discovery.type", "single-node")
            .with_env_var("xpack.security.enabled", "false")
            .with_env_var("ES_JAVA_OPTS", "-Xms512m -Xmx512m")
            .with_ready_conditions(vec![WaitFor::message_on_either_std("started"), WaitFor::seconds(2)]),
        9200,
    )
    .await
}
