//! tests/health_check.rs

use std::net::TcpListener;
use sqlx::{PgConnection, Connection};
use zero2prod::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() {
  // starting up app
  let address = spawn_app();

  // creating client to make requests to app
  let client = reqwest::Client::new();

  // Sending request
  let response = client
    .get(&format!("{}/health_check", &address))
    .send()
    .await
    .expect("Failed to execute request.");

  // Checking response
  assert!(response.status().is_success());
  assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
  // create a listener that looks for open port on loclahost
  let listener = TcpListener::bind("127.0.0.1:0")
    .expect("Failed to bind random port");
  // find currently used port number
  let port = listener.local_addr().unwrap().port();
  let server = zero2prod::startup::run(listener).expect("Failed to bind address.");
  std::mem::drop(tokio::spawn(server)); 
  format!("http://127.0.0.1:{}", port) 
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
  // Start the application
  let app_address = spawn_app();
  let configuration = get_configuration().expect("Failed to read configuration");

  let connection_strig = configuration.database.connection_string();
  let mut connection = PgConnection::connect(&connection_strig)
    .await
    .expect("Failed to connect to Postgres");

  // create client
  let client = reqwest::Client::new();

  // Performing test
  let body = "name=le%20Guin&email=ursula_le_guin%40gmail.com";
  let response = client
    .post(&format!("{}/subscriptions", &app_address))
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(body)
    .send()
    .await
    .expect("Failed to execute request.");

  // Check result
  assert_eq!(200, response.status().as_u16());

  let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
    .fetch_one(&mut connection)
    .await
    .expect("Failed to fetch saved subscription.");

  assert_eq!(saved.email, "ursula_le_guin@gmail.com");
  assert_eq!(saved.name, "le Guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
  // Start the application
  let app_address = spawn_app();

  // create client
  let client = reqwest::Client::new();

  // test cases
  let test_cases = vec![
    ("name=le%20guin", "missing the email"), 
    ("email=ursula_le_guin%40gmail.com", "missing the name"),
    ("", "missing both name and email")
  ];

  // Going through all the test cases
  for (invalid_body, error_message) in test_cases {
    // Performing test
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to execute request");

    // Checking results
    assert_eq!(
      400,
      response.status().as_u16(),
      "The API did not fail with 400 Bad Request when the payload was {}.",
      error_message
    );
  }
}