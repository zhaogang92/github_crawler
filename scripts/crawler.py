# coding=utf-8
"""
Collect commits with specific keywords.
"""
import os
import time
from github import Github
from tqdm import tqdm


github_token = os.environ["GH_TOKEN"]
if github_token is None or github_token == "":
    print("Please specify GH_TOKEN env variable")
    exit(-1)
g = Github(login_or_token=github_token, per_page=100)


def get_top_k(k=100):
    page = g.search_repositories(
        query="",
        sort="stars",
        order="desc",
        language="rust",
        size="1000..10000",
        stars=">=100",
        pushed=">2020-01-01"
        )
    repos = []
    for repo in page:
        repos.append(repo)
        if len(repos) == k:
            break
    return repos


def get_commits(repos, k=100):
    commits = []
    for repo in tqdm(repos):
        time.sleep(3)   # rate limit 30/min
        page = g.search_commits(
            # query="atomicity violation committer-date:>2019-01-01",
            query="race condition committer-date:>2019-01-01",
            repo=repo,
        )
        for commit in page:
            commits.append(commit.html_url)
    return commits


# def get_pulls(repos):
#     pulls = []
#     for r in tqdm(repos):
#         time.sleep(3)
#         repo = g.get_repo(r) 
#         page = repo.get_pulls(sort="created")


def load_repos():
    if not os.path.exists("./repos.txt"):
        repos = get_top_k(1000)
        with open("./repos.txt", "w") as out:
            out.writelines([repo.full_name + "\n" for repo in repos])
    return list(map(lambda line: line.strip(), open("./repos.txt", "r").readlines()))


if __name__ == '__main__':
    repos = load_repos()
    commits = get_commits(repos)
    with open("./commits.txt", "w") as out:
        out.writelines([c + "\n" for c in commits])
