//! Email service module for sending password reset emails

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;
use log;

/// Email service for sending password reset emails
pub struct EmailService {
    smtp_transport: SmtpTransport,
    from_email: String,
    from_name: String,
}

impl EmailService {
    /// Creates a new email service instance
    pub fn new() -> Result<Self, String> {
        let smtp_host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST not set".to_string())?;
        let smtp_port = env::var("SMTP_PORT")
            .map_err(|_| "SMTP_PORT not set".to_string())?
            .parse::<u16>()
            .map_err(|_| "Invalid SMTP_PORT".to_string())?;
        
        let smtp_username = env::var("SMTP_USERNAME").unwrap_or_default();
        let smtp_password = env::var("SMTP_PASSWORD").unwrap_or_default();
        let from_email = env::var("SMTP_FROM_EMAIL").map_err(|_| "SMTP_FROM_EMAIL not set".to_string())?;
        let from_name = env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "PassQ".to_string());

        // Build SMTP transport - use unencrypted connection for MailHog
        let mut transport_builder = SmtpTransport::builder_dangerous(&smtp_host)
            .port(smtp_port);

        // Add authentication if credentials are provided
        if !smtp_username.is_empty() && !smtp_password.is_empty() {
            let creds = Credentials::new(smtp_username, smtp_password);
            transport_builder = transport_builder.credentials(creds);
        }

        let smtp_transport = transport_builder.build();

        log::info!("Email service initialized with SMTP host: {}:{}", smtp_host, smtp_port);

        Ok(EmailService {
            smtp_transport,
            from_email,
            from_name,
        })
    }

    /// Sends a password reset email
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        username: &str,
        reset_token: &str,
    ) -> Result<(), String> {
        let reset_url = format!("http://localhost/reset-password?token={}", reset_token);
        
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Password Reset - PassQ</title>
                <style>
                    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
                    body {{ 
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
                        line-height: 1.6; 
                        color: #1f2937;
                        background-color: #f3f4f6;
                        padding: 20px;
                    }}
                    .email-container {{ 
                        max-width: 600px; 
                        margin: 0 auto; 
                        background-color: #ffffff;
                        border-radius: 16px;
                        overflow: hidden;
                        box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
                        border: 3px solid #000000;
                    }}
                    .header {{ 
                        background: #000000;
                        color: white; 
                        padding: 40px 30px;
                        text-align: center;
                        position: relative;
                    }}
                    .header::after {{
                        content: '';
                        position: absolute;
                        bottom: -3px;
                        left: 0;
                        right: 0;
                        height: 3px;
                        background-color: #000000;
                    }}
                    .header h1 {{ 
                        font-size: 32px;
                        font-weight: 900;
                        letter-spacing: 2px;
                        text-transform: uppercase;
                        margin: 0;
                        text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
                    }}
                    .header .subtitle {{
                        font-size: 14px;
                        opacity: 0.9;
                        margin-top: 8px;
                        font-weight: 600;
                        letter-spacing: 1px;
                        text-transform: uppercase;
                    }}
                    .content {{ 
                        padding: 40px 30px;
                        background-color: #ffffff;
                    }}
                    .content h2 {{
                        font-size: 24px;
                        font-weight: 800;
                        color: #1f2937;
                        margin-bottom: 20px;
                        text-transform: uppercase;
                        letter-spacing: 1px;
                    }}
                    .content p {{
                        margin-bottom: 16px;
                        color: #4b5563;
                        font-size: 16px;
                        line-height: 1.7;
                    }}
                    .greeting {{
                        font-size: 18px;
                        font-weight: 700;
                        color: #1f2937;
                        margin-bottom: 24px;
                    }}
                    .button-container {{
                        text-align: center;
                        margin: 32px 0;
                    }}
                    .button {{ 
                        display: inline-block;
                        padding: 16px 32px;
                        background: #8B0000;
                        color: white;
                        text-decoration: none;
                        border-radius: 12px;
                        font-weight: 800;
                        font-size: 16px;
                        text-transform: uppercase;
                        letter-spacing: 1px;
                        border: 3px solid #000000;
                        box-shadow: 4px 4px 0px #000000;
                        transition: all 0.2s ease;
                    }}
                    .button:hover {{
                        transform: translate(-2px, -2px);
                        box-shadow: 6px 6px 0px #000000;
                    }}
                    .link-section {{
                        background-color: #f9fafb;
                        border: 2px solid #e5e7eb;
                        border-radius: 8px;
                        padding: 20px;
                        margin: 24px 0;
                        text-align: center;
                    }}
                    .link-section p {{
                        margin-bottom: 12px;
                        font-size: 14px;
                        color: #6b7280;
                    }}
                    .reset-link {{
                        word-break: break-all;
                        color: #1f2937;
                        text-decoration: underline;
                        font-weight: 600;
                    }}
                    .warning {{
                        background-color: #fef3c7;
                        border: 2px solid #f59e0b;
                        border-radius: 8px;
                        padding: 16px;
                        margin: 24px 0;
                        text-align: center;
                    }}
                    .warning p {{
                        color: #92400e;
                        font-weight: 700;
                        margin: 0;
                        font-size: 14px;
                        text-transform: uppercase;
                        letter-spacing: 0.5px;
                    }}
                    .footer {{ 
                        background-color: #f9fafb;
                        padding: 30px;
                        text-align: center;
                        border-top: 3px solid #000000;
                    }}
                    .footer p {{
                        color: #6b7280;
                        font-size: 12px;
                        margin: 0;
                        font-weight: 500;
                        text-transform: uppercase;
                        letter-spacing: 0.5px;
                    }}
                    @media (max-width: 600px) {{
                        body {{ padding: 10px; }}
                        .email-container {{ border-radius: 12px; }}
                        .header {{ padding: 30px 20px; }}
                        .header h1 {{ font-size: 24px; }}
                        .content {{ padding: 30px 20px; }}
                        .button {{ padding: 14px 24px; font-size: 14px; }}
                    }}
                </style>
            </head>
            <body>
                <div class="email-container">
                    <div class="header">
                        <h1>PassQ</h1>
                        <div class="subtitle">Password Manager</div>
                    </div>
                    <div class="content">
                        <h2>🔐 Password Reset Request</h2>
                        <p class="greeting">Hello {},</p>
                        <p>We received a request to reset your password for your PassQ account. If you didn't make this request, you can safely ignore this email.</p>
                        <p>To reset your password, click the button below:</p>
                        
                        <div class="button-container">
                            <a href="{}" class="button">Reset Password</a>
                        </div>
                        
                        <div class="link-section">
                            <p>Or copy and paste this link into your browser:</p>
                            <a href="{}" class="reset-link">{}</a>
                        </div>
                        
                        <div class="warning">
                            <p>⚠️ This link will expire in 15 minutes for security reasons</p>
                        </div>
                    </div>
                    <div class="footer">
                        <p>This email was sent by PassQ Password Manager.<br>If you didn't request this email, please ignore it.</p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            username, reset_url, reset_url, reset_url
        );

        let text_body = format!(
            "Hello {},\n\n\
            We received a request to reset your password for your PassQ account.\n\n\
            To reset your password, visit this link:\n\
            {}\n\n\
            This link will expire in 15 minutes for security reasons.\n\n\
            If you didn't request this password reset, you can safely ignore this email.\n\n\
            Best regards,\n\
            The PassQ Team",
            username, reset_url
        );

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse().map_err(|e| format!("Invalid from address: {}", e))?)
            .to(to_email.parse().map_err(|e| format!("Invalid to address: {}", e))?)
            .subject("Password Reset - PassQ")
            .header(ContentType::TEXT_HTML)
            .body(html_body)
            .map_err(|e| format!("Failed to build email: {}", e))?;

        match self.smtp_transport.send(&email) {
            Ok(_) => {
                log::info!("Password reset email sent successfully to: {}", to_email);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to send password reset email to {}: {}", to_email, e);
                Err(format!("Failed to send email: {}", e))
            }
        }
    }

    /// Test email connectivity
    pub async fn test_connection(&self) -> Result<(), String> {
        match self.smtp_transport.test_connection() {
            Ok(true) => {
                log::info!("SMTP connection test successful");
                Ok(())
            }
            Ok(false) => {
                log::error!("SMTP connection test failed");
                Err("SMTP connection test failed".to_string())
            }
            Err(e) => {
                log::error!("SMTP connection error: {}", e);
                Err(format!("SMTP connection error: {}", e))
            }
        }
    }
}