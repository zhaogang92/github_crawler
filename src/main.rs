use octocrab::models;
use std::{env, io::{BufRead, Write}, time::Duration};
use std::fs::{File, OpenOptions};
use std::io;

async fn get_top_k(k: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if let Ok(repo_file) = File::open("repos.txt") {
        return Ok(io::BufReader::new(repo_file).lines().take(k).flatten().collect());
    }
    let mut repos = Vec::new();
    let token = env::var("GH_TOKEN").expect("Please specify the GH_TOKEN env variable");
    let octocrab = octocrab::OctocrabBuilder::new()
        .personal_token(token)
        .build().unwrap();
    if let Ok(mut page) = octocrab
        .search()
        .repositories("language:rust size:1000..10000 stars:>=100 pushed:>2020-01-01")
        .sort("stars")
        .order("desc")
        .send()
        .await {
        repos.extend(page.take_items().into_iter().map(|it| it.full_name));
        while let Ok(Some(mut page)) = octocrab.get_page::<models::Repository>(&page.next).await {
            if repos.len() > k {
                break;
            }
            repos.extend(page.take_items().into_iter().map(|it| it.full_name));
        }
    }
    let repo_file = OpenOptions::new().write(true).create(true).open("repos.txt")?;
    let mut writer = io::LineWriter::new(repo_file);
    writer.write_all(repos.join("\n").as_bytes())?;
    Ok(repos)
}

async fn get_commits(k: usize, repos: &[String], keywords: &str) -> Result<Vec<models::repos::Commit>, Box<dyn std::error::Error>> {
    let token = env::var("GH_TOKEN").expect("Please specify the GH_TOKEN env variable");
    let octocrab = octocrab::OctocrabBuilder::new()
        .add_preview("cloak")
        .personal_token(token)
        .build().unwrap();
    let mut commits = Vec::new();
    for r in repos {
        // The rate limit is 30/min for basic auth
        tokio::time::sleep(Duration::from_secs(3)).await;
        println!("Search repo {}", r);
        // let query = format!("{} repo:{} committer-date:>2019-01-01", keywords, r.full_name);
        let query = format!("{} repo:{}", keywords, r);
        match octocrab
            .search()
            .commits(query.as_str())
            .sort("committer-date")
            .order("desc")
            .send()
            .await {
            Ok(mut page) => {
                commits.extend(page.take_items()); 
                while let Ok(Some(mut page)) = octocrab.get_page::<models::repos::Commit>(&page.next).await {
                    if commits.len() > k {
                        break;
                    }
                    commits.extend(page.take_items());
                    tokio::time::sleep(Duration::from_secs(3)).await;
                }
            }
            Err(err) => {
                println!("Failed: {:#?}", err)
            } 
        }
    }
    Ok(commits)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let output_path = args[1].as_str();
    let repos = get_top_k(10).await.unwrap();
    let commits = get_commits(1000, repos.as_slice(), "fix").await?;
    let commit_file = OpenOptions::new().write(true).create(true).open(output_path)?;
    io::LineWriter::new(commit_file).write_all(
        commits.iter().map(|it| it.html_url.as_ref().map(|u| u.as_str())).flatten().collect::<Vec<&str>>().join("\n").as_bytes())?;
    for (idx, commit) in commits.iter().enumerate() {
        println!("{}, {}", idx, commit.url.as_ref().unwrap());
    }
    Ok(())
}