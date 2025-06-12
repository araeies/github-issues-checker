use futures::future::*;
use reqwest::header::{ HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use dotenv::dotenv;

#[derive(Debug, Serialize, Deserialize)]
struct PullRequest {}

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    number: usize,
    title: String,
    pull_request: Option<PullRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    login: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IssueReaction {
    content: String,
    user: User,
}

fn contruct_new_url(headers: &HeaderMap) -> Option<String> {
    headers.get("link").and_then(|link_header| {
        link_header.to_str().ok()
            .and_then(|link_value|{
                link_value.contains("rel=\"next\"")
                    .then(|| {
                        link_value.split(";")
                            .collect::<Vec<&str>>()
                            .get(0)
                            .expect("Could not find new url with page")
                            .trim_start_matches("<")
                            .trim_end_matches(">")
                            .to_string()
                    })
            
        })
    })


}

fn get_issues_wrapper (url: Option<String>) -> BoxFuture<'static, Vec<Issue>> {
    Box::pin(get_issues(url))
}

async fn get_issues(url: Option<String>) -> Vec<Issue> {
    let token = std::env::var("GITHUB_PAT").expect("Expected GITHUB_PAT in env file");
    let request_url = url.unwrap_or(format!("https://api.github.com/repos/{owner}/{repo}/issues?state=open&page=1",
     owner ="araeies",
    repo = "password-gen"));
    let client = reqwest::Client::new();
    let response = client
    .get(&request_url)
    .header(AUTHORIZATION, format!("Bearer {token}", token = token))
    .header(USER_AGENT, "rust web-api")
    .header(ACCEPT, "application/vnd.github+json")
    .send()
    .await;

    let response = match response{
        Ok(res) if res.status().is_success() => res,
        _ => return Vec::new()
    };

    let new_url = contruct_new_url(response.headers());
     println!("Issues link: {:?}",new_url);

    let issues = response
    .json::<Vec<Issue>>()
    .await
    .expect("Something went wrong while parsing")
    .into_iter()
    .filter(|issue| issue.pull_request.is_none())
    .collect::<Vec<Issue>>();


    if let Some(new_url) = new_url {
        let more_issues = get_issues_wrapper(Some(new_url) ).await;
        return issues.into_iter().chain(more_issues).collect();
        
    }
    // Return the issues
    issues          
    
}

async fn get_issues_reactions(issue_id: usize, ) -> Vec<IssueReaction> {
    let token = std::env::var("GITHUB_PAT").expect("Expected GITHUB_PAT in env file");
    let request_url = format!("https://api.github.com/repos/{owner}/{repo}/issues/{issue_id}/reactions",
     owner ="araeies",
    repo = "password-gen",
    issue_id = issue_id);
    let client = reqwest::Client::new();
    let response = client
    .get(&request_url)
    .header(AUTHORIZATION, format!("Bearer {token}", token = token))
    .header(USER_AGENT, "rust web-api")
    .header(ACCEPT, "application/vnd.github+json")
    .send()
    .await
    .expect("Something went wrong while parsing");

    let resolved_response = response
    .json::<Vec<IssueReaction>>()
    .await
    .expect("Something went wrong while parsing");

    resolved_response
}
    


#[tokio::main]
async fn main() {

    dotenv().ok();
    println!("Starting the GitHub Issues Fetcher...");
    // Fetch issues from the GitHub repository
    let issues = get_issues(None).await;
    println!("Amount of Issues: {:?}", issues.len());

    for issue in &issues {
        let reactions = get_issues_reactions(issue.number).await;
        println!("Issue: {}", issue.title);
        println!("Reactions: {:?}", reactions); 
}
}



