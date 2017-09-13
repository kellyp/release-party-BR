extern crate reqwest;
extern crate serde_json;

use self::reqwest::header::{Authorization, Link, UserAgent};
use self::reqwest::{Error, Response, Url};

use std::io::Read;
use std::collections::HashMap;
use std::{thread, time};

static USERAGENT: &'static str = "release-party-br";

#[derive(Deserialize, Debug)]
pub struct GithubRepo {
    id: i32,
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct CompareCommitsResponse {
    pub status: String,
    pub behind_by: i32,
}

#[derive(Deserialize, Debug)]
pub struct GithubPullRequest {
    id: i32,
    pub url: String,
    pub html_url: String,
    pub head: Commit,
    pub base: Commit,
}

#[derive(Deserialize, Debug)]
pub struct Commit {
    pub sha: String,
    pub label: String,
}

pub fn is_release_up_to_date_with_master(repo_url: &str, token: &str, client: &reqwest::Client) -> bool {
    let repo_pr_url = format!("{}/{}/{}...{}", repo_url, "compare", "master", "release");
    let url = match Url::parse(&repo_pr_url) {
        Ok(new_url) => new_url,
        Err(e) => {
            println!("Couldn't create url for compare page: {}", e);
            return true;
        }
    };
    let mut res = match client
        .get(url.clone())
        .expect("Couldn't make a request builder for comparison page url")
        .header(UserAgent::new(USERAGENT.to_string()))
        .header(Authorization(format!("token {}", token)))
        .send()
    {
        Ok(response) => response,
        Err(e) => {
            println!("Error in request to github for compare page: {}", e);
            return true;
        }
    };

    let mut buffer = String::new();

    match res.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error checking commit diff for {}: {}", repo_url, e),
    }

    let commits_diff: CompareCommitsResponse = match serde_json::from_str(&buffer) {
        Ok(compare_response) => compare_response,
        Err(_) => return true,
    };

    if commits_diff.behind_by > 0 {
        return false;
    }

    true
}

fn response_has_a_next_link(response_headers: &reqwest::header::Headers) -> bool {
    if response_headers.get::<Link>().is_none() {
        return false;
    }
    for link in response_headers
        .get::<Link>()
        .expect("links in has next")
        .values()
    {
        let next_link = link.rel();
        match next_link {
            Some(v) => {
                let link_something = v.first().expect("should have value in links headers");
                if link_something == &reqwest::header::RelationType::Next {
                    return true;
                }
            }
            None => return false,
        }
    }
    false
}

// Expects caller to check to ensure the `next` link is present
fn response_next_link(response_headers: &reqwest::header::Headers) -> Result<Url, String> {
    for link in response_headers
        .get::<Link>()
        .expect("links in response next")
        .values()
    {
        let next_link = link.rel();
        match next_link {
            Some(v) => {
                let link_something = v.first().expect("should have value in links headers");
                if link_something == &reqwest::header::RelationType::Next {
                    let uri = Url::parse(link.link()).expect("Should have been able to parse the nextlink");
                    return Ok(uri);
                }
            }
            None => (),
        }
    }
    Err("Couldn't find a next link: does it exist?".to_string())
}

pub fn get_repos_at(repos_url: &str, token: &str, client: &reqwest::Client) -> Result<Vec<GithubRepo>, String> {
    // We need to pass in the URL from the link headers to github API docs.
    // We'll construct it this first time.
    let url = match Url::parse_with_params(repos_url, &[("per_page", "50")]) {
        Ok(new_url) => new_url,
        Err(e) => return Err(format!("Couldn't parse uri {:?} : {:?}", repos_url, e)),
    };
    let mut response = get_repos_at_url(url, token, client).expect("request failed");

    let mut buffer = String::new();
    match response.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error reading response from github when getting repo list: {}", e),
    }
    let mut repos = repo_list_from_string(&buffer).expect("expected repos");

    if response_has_a_next_link(response.headers()) {
        loop {
            thread::sleep(time::Duration::from_millis(1500));

            let paging_url = response_next_link(response.headers()).expect("a thing");
            response = get_repos_at_url(paging_url, token, client).expect("request failed");

            buffer = String::new();
            match response.read_to_string(&mut buffer) {
                Ok(_) => (),
                Err(e) => println!("error reading response from github when getting repo list: {}", e),
            }
            repos.append(&mut repo_list_from_string(&buffer).expect("expected repos"));
            if !response_has_a_next_link(response.headers()) {
                break;
            }
        }
    }
    println!("Number of repos to check: {:?}", repos.len());
    Ok(repos)
}

fn get_repos_at_url(url: reqwest::Url, token: &str, client: &reqwest::Client) -> Result<Response, Error> {
    client
        .get(url)
        .expect("Couldn't make a request builder for repos page url")
        .header(UserAgent::new(USERAGENT.to_string()))
        .header(Authorization(format!("token {}", token)))
        .send()
}

fn repo_list_from_string(json_str: &str) -> Result<Vec<GithubRepo>, String> {
    // This looks a bit weird due to supplying type hints to deserialize:
    let _: Vec<GithubRepo> = match serde_json::from_str(json_str) {
        Ok(v) => return Ok(v),
        Err(e) => return Err(format!("Couldn't deserialize repos from github: {}", e)),
    };
}

pub fn existing_release_pr_location(repo: &GithubRepo, token: &str, client: &reqwest::Client) -> Option<String> {
    let repo_pr_url = format!("{}/{}", repo.url, "pulls");
    let url = match Url::parse_with_params(&repo_pr_url, &[("head", "master"), ("base", "release")]) {
        Ok(new_url) => new_url,
        Err(e) => {
            println!("Couldn't create url for existing pr location: {}", e);
            return None;
        }
    };
    let mut res = match client
        .get(url)
        .expect("Couldn't make a request builder for existing PRs page url")
        .header(UserAgent::new(USERAGENT.to_string()))
        .header(Authorization(format!("token {}", token)))
        .send()
    {
        Ok(response) => response,
        Err(e) => {
            println!("Error in request to github for existing PR location: {}", e);
            return None;
        }
    };

    let mut buffer = String::new();

    match res.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error finding existing pr for {}: {}", repo.name, e),
    }

    let pull_reqs: Vec<GithubPullRequest> = match serde_json::from_str(&buffer) {
        Ok(v) => v,
        Err(_) => Vec::new(),
    };

    if !pull_reqs.is_empty() {
        return Some(pull_reqs[0].html_url.clone());
    }

    None
}

// Try to create the release PR and return the URL of it:
pub fn create_release_pull_request(repo: &GithubRepo, token: &str, client: &reqwest::Client) -> Result<String, String> {
    let mut pr_body = HashMap::new();
    pr_body.insert("title", "automated release partay");
    pr_body.insert("head", "master");
    pr_body.insert("base", "release");

    let repo_pr_url = format!("{}/{}", repo.url, "pulls");
    let mut res = match client
        .post(&repo_pr_url)
        .expect("Couldn't make a request builder for creating PR url")
        .header(UserAgent::new(USERAGENT.to_string()))
        .header(Authorization(format!("token {}", token)))
        .json(&pr_body)
        .expect("Couldn't make the JSON payload for creating a PR")
        .send()
    {
        Ok(response) => response,
        Err(e) => return Err(format!("Error in request to github creating new PR: {}", e)),
    };

    if res.status().is_success() {
        let mut buffer = String::new();
        match res.read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(e) => println!("error reading response after creating new release PR for {}: {}", repo.name, e),
        }
        let pull_req: GithubPullRequest = match serde_json::from_str(&buffer) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Couldn't deserialize create pull req response for {}: {}",
                    repo.name,
                    e
                ))
            }
        };
        return Ok(pull_req.html_url);
    }
    // 422 unprocessable means it's there already
    // 422 unprocessable also means the branch is up to date

    Err("Release branch already up to date?".to_string())
}
