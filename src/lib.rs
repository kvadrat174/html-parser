use bindgen_prelude::Buffer;
use napi::{*};
use std::collections::HashMap;
use scraper::{Html, Selector};
use kuchiki::traits::*;
use kuchiki::parse_html;
use regex::Regex;
use std::str;

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct ReplacedLink {
    pub href: String,
    pub tracked: String,
}

#[napi(object)]
pub struct ReplacedUser {
  pub name: Option<String>,
  pub surname: Option<String>,
}

#[napi]
pub fn replace_handlebars_tokens(buffer: Buffer, user: Option<ReplacedUser>) -> Result<Buffer> {
  // Convert buffer to string
  let html_content = str::from_utf8(&buffer)
      .map_err(|e| napi::Error::from_reason(format!("Failed to convert buffer to string: {}", e)))?;

  // Replace {{ user.name }} if user is defined
  let replaced_content = if let Some(user) = user {
      let mut replaced = html_content.to_string();

      if let Some(name) = user.name {
          let replacement_pattern_name = r"\{\{\s*user\.name\s*\}\}";
          let re_name = Regex::new(replacement_pattern_name)
              .map_err(|e| napi::Error::from_reason(format!("Regex error: {}", e)))?;
          replaced = re_name.replace_all(&replaced, name.as_str()).to_string();
      }

      if let Some(surname) = user.surname {
          let replacement_pattern_surname = r"\{\{\s*user\.surname\s*\}\}";
          let re_surname = Regex::new(replacement_pattern_surname)
              .map_err(|e| napi::Error::from_reason(format!("Regex error: {}", e)))?;
          replaced = re_surname.replace_all(&replaced, surname.as_str()).to_string();
      }

      replaced
  } else {
      html_content.to_string()
  };

  // Convert replaced content back to bytes and create a new Buffer
  let replaced_bytes = replaced_content.into_bytes();
  let modified_buffer = Buffer::from(replaced_bytes);

  Ok(modified_buffer)
}

#[napi]
pub fn find_all_hrefs(buffer: Buffer) -> Result<Vec<String>> {
    let html_content = String::from_utf8(buffer.to_vec())
        .map_err(|e| napi::Error::from_reason(format!("Failed to convert buffer to string: {}", e)))?;

    let document = Html::parse_document(&html_content);

    // Create a selector for <a> tags
    let selector = Selector::parse("a").map_err(|e| napi::Error::from_reason(format!("Failed to create selector: {:?}", e)))?;

    // Find all <a> tags and extract href attributes
    let mut hrefs = Vec::new();
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            hrefs.push(href.to_string());
        }
    }

    Ok(hrefs)
}

#[napi]
pub fn find_handlebars_tokens(buffer: Buffer, user_id: Option<String>) -> Result<Vec<String>> {
    // Convert buffer to string
    let html_content = String::from_utf8(buffer.to_vec())
        .map_err(|e| {
            napi::Error::from_reason(format!("Failed to convert buffer to string: {}", e))
        })?;

    let re = Regex::new(r"\{\{\s*([^\s\}]+)\s*\}\}").unwrap();
    let mut tokens = Vec::new();

    for capture in re.captures_iter(&html_content) {
        if let Some(token) = capture.get(1) {
            tokens.push(token.as_str().to_string());
        }
    }

    Ok(tokens)
}

#[napi]
pub fn add_pre_header_and_links(
    buffer: Buffer,
    links: Vec<ReplacedLink>,
    open_link: String,
    header: Option<String>,
    user_id: Option<String>
) -> Result<Buffer> {
    let html_content = String::from_utf8(buffer.to_vec())
        .map_err(|e| napi::Error::from_reason(format!("Failed to convert buffer to string: {}", e)))?;
    
    // Parse the HTML content
    let document = parse_html().one(html_content);

    // Add pre-header if provided
    if let Some(header_content) = header {
        let pre_header = format!(
            "<div style=\"font-size:0px;line-height:1px;mso-line-height-rule:exactly;display:none;max-width:0px;max-height:0px;opacity:0;overflow:hidden;mso-hide:all;\">{}</div>",
            header_content
        );
        let pre_header_node = parse_html().one(pre_header).select_first("div").unwrap();
        if let Some(body) = document.select_first("body").ok() {
          body.as_node().insert_before(pre_header_node.as_node().clone());
        }
    }

    // Add tracking link
    let tracking_link = format!(
        "<img src=\"{}\" alt style=\"display:block;border:0;outline:none;text-decoration:none;\" width=\"1\" height=\"1\">",
        open_link
    );
    let tracking_node = parse_html().one(tracking_link).select_first("img").unwrap();
    if let Some(body) = document.select_first("body").ok() {
        body.as_node().append(tracking_node.as_node().clone());
    }

    // Set up the replacements
    let excluded_links = vec!["{{unsubscribeLink}}", "{{telegramLink}}"];
    let mut link_replacements = HashMap::new();
    if let Some(id) = user_id {
        link_replacements.insert("userId".to_string(), id);
    }

    // Replace hrefs
    let mut elements_to_update = Vec::new();
    for css_match in document.select("a").unwrap() {
        let link_node = css_match.as_node();
        let attributes = link_node.as_element().unwrap().attributes.borrow();
        if let Some(href) = attributes.get("href") {
            if !excluded_links.contains(&href) {
                let mut new_href = href.to_string();
                if includes_string_tokens(&new_href) {
                    new_href = replace_tokens(&new_href, &link_replacements);
                }
                if let Some(upd_href) = links.iter().find(|r| r.href == new_href) {
                    elements_to_update.push((link_node.clone(), upd_href.tracked.clone()));
                }
            }
        }
    }

    // Apply updates
    for (element, updated_href) in elements_to_update {
        let mut attributes = element.as_element().unwrap().attributes.borrow_mut();
        attributes.insert("href", updated_href);
    }

    let result_html = document.to_string();
    Ok(Buffer::from(result_html.as_bytes()))
}

fn includes_string_tokens(link: &str) -> bool {
    let mail_replacements = vec!["userId"]; // Add other tokens if needed
    let regex_str = format!(r"\{{\{{({})\}}\}}", mail_replacements.join("|"));
    let regex = Regex::new(&regex_str).unwrap();
    regex.is_match(link)
}

fn replace_tokens(link: &str, replacements: &HashMap<String, String>) -> String {
    let mut result = link.to_string();
    for (key, value) in replacements {
        let token = format!("{{{{{}}}}}", key);
        result = result.replace(&token, value);
    }
    result
}
