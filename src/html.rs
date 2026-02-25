use crate::{error::ServiceError, themes::THEME_NAMES};

pub fn missing_username_page(base_path: &str) -> String {
    let themes = THEME_NAMES.join(", ");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>GitHub Profile Trophy</title>
  <style>
    body {{ font-family: Arial, sans-serif; margin: 0; background: #f4f4f4; color: #222; }}
    section {{ width: min(860px, 92vw); margin: 24px auto; }}
    .card {{ background: #fff; border-radius: 8px; padding: 20px; margin-bottom: 16px; box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08); }}
    code {{ background: #f0f0f0; padding: 2px 6px; border-radius: 4px; }}
    input {{ width: 100%; box-sizing: border-box; padding: 10px 12px; margin: 8px 0 16px; border: 1px solid #d0d7de; border-radius: 6px; }}
    button {{ padding: 10px 14px; border: none; border-radius: 6px; background: #24292f; color: #fff; cursor: pointer; }}
    button:hover {{ background: #3d444d; }}
    .muted {{ color: #57606a; font-size: 14px; }}
  </style>
</head>
<body>
  <section>
    <div class="card">
      <h2>"username" is a required query parameter</h2>
      <p>URL example: <code>{base_path}?username=USERNAME</code></p>
      <p class="muted">Example themes: {themes}</p>
    </div>
    <div class="card">
      <h2>Generate Trophy</h2>
      <form action="{base_path}" method="get">
        <label for="username">GitHub Username</label>
        <input id="username" name="username" type="text" placeholder="Ex. h-sumiya" required />

        <label for="theme">Theme (optional)</label>
        <input id="theme" name="theme" type="text" placeholder="Ex. onedark" value="default" />

        <button type="submit">Get Trophies</button>
      </form>
    </div>
  </section>
</body>
</html>"#
    )
}

pub fn error_page(error: &ServiceError) -> String {
    let (status, message, detail) = match error {
        ServiceError::RateLimit => (419, "Rate Limit Exceeded", "Please retry later."),
        ServiceError::NotFound => (
            404,
            "Not Found",
            "Sorry, the user you are looking for was not found.",
        ),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>GitHub Profile Trophy</title>
  <style>
    body {{ font-family: Arial, sans-serif; margin: 0; background: #f4f4f4; color: #222; }}
    section {{ width: min(760px, 92vw); margin: 48px auto; }}
    .card {{ background: #fff; border-radius: 8px; padding: 24px; box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08); }}
  </style>
</head>
<body>
  <section>
    <div class="card">
      <h1>{status} - {message}</h1>
      <p>{detail}</p>
    </div>
  </section>
</body>
</html>"#
    )
}
