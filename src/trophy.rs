use std::collections::HashSet;

use crate::models::UserInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rank {
    Secret,
    Sss,
    Ss,
    S,
    Aaa,
    Aa,
    A,
    B,
    C,
    Unknown,
}

impl Rank {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Secret => "SECRET",
            Self::Sss => "SSS",
            Self::Ss => "SS",
            Self::S => "S",
            Self::Aaa => "AAA",
            Self::Aa => "AA",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::Unknown => "?",
        }
    }

    pub fn first_letter(self) -> &'static str {
        match self {
            Self::Unknown => "?",
            Self::Secret => "S",
            Self::Sss => "S",
            Self::Ss => "S",
            Self::S => "S",
            Self::Aaa => "A",
            Self::Aa => "A",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }
}

pub const RANK_ORDER: [Rank; 10] = [
    Rank::Secret,
    Rank::Sss,
    Rank::Ss,
    Rank::S,
    Rank::Aaa,
    Rank::Aa,
    Rank::A,
    Rank::B,
    Rank::C,
    Rank::Unknown,
];

#[derive(Debug, Clone, Copy)]
pub struct RankCondition {
    pub rank: Rank,
    pub message: &'static str,
    pub required_score: i64,
}

#[derive(Debug, Clone)]
pub struct Trophy {
    pub rank_condition: Option<RankCondition>,
    pub rank: Rank,
    pub top_message: String,
    pub bottom_message: String,
    pub title: &'static str,
    pub filter_titles: &'static [&'static str],
    pub hidden: bool,
    score: i64,
    rank_conditions: &'static [RankCondition],
}

impl Trophy {
    fn new(
        score: i64,
        title: &'static str,
        filter_titles: &'static [&'static str],
        hidden: bool,
        rank_conditions: &'static [RankCondition],
        bottom_override: Option<&'static str>,
    ) -> Self {
        let mut trophy = Self {
            rank_condition: None,
            rank: Rank::Unknown,
            top_message: "Unknown".to_string(),
            bottom_message: abridge_score(score),
            title,
            filter_titles,
            hidden,
            score,
            rank_conditions,
        };

        trophy.set_rank();

        if let Some(bottom) = bottom_override {
            trophy.bottom_message = bottom.to_string();
        }

        trophy
    }

    fn set_rank(&mut self) {
        let mut sorted = self.rank_conditions.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|condition| rank_order_index(condition.rank));

        if let Some(condition) = sorted
            .into_iter()
            .find(|condition| self.score >= condition.required_score)
        {
            self.rank = condition.rank;
            self.rank_condition = Some(*condition);
            self.top_message = condition.message.to_string();
        }
    }

    pub fn calculate_next_rank_percentage(&self) -> f64 {
        if self.rank == Rank::Unknown {
            return 0.0;
        }

        let current_index = rank_order_index(self.rank);
        if current_index == 0 || self.rank == Rank::Sss {
            return 1.0;
        }

        let current_condition = match self.rank_condition {
            Some(condition) => condition,
            None => return 1.0,
        };

        let next_rank = RANK_ORDER[current_index - 1];
        let next_condition = match self
            .rank_conditions
            .iter()
            .find(|condition| condition.rank == next_rank)
            .copied()
        {
            Some(condition) => condition,
            None => return 1.0,
        };

        let distance = next_condition.required_score - current_condition.required_score;
        if distance <= 0 {
            return 1.0;
        }

        let progress = self.score - current_condition.required_score;
        (progress as f64 / distance as f64).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct TrophyList {
    trophies: Vec<Trophy>,
}

impl TrophyList {
    pub fn new(user_info: &UserInfo) -> Self {
        let mut trophies = vec![
            total_star_trophy(user_info.total_stargazers),
            total_commit_trophy(user_info.total_commits),
            total_follower_trophy(user_info.total_followers),
            total_issue_trophy(user_info.total_issues),
            total_pull_request_trophy(user_info.total_pull_requests),
            total_repository_trophy(user_info.total_repositories),
            total_reviews_trophy(user_info.total_reviews),
        ];

        let is_all_s_rank = if trophies
            .iter()
            .all(|trophy| trophy.rank.as_str().starts_with('S'))
        {
            1
        } else {
            0
        };

        trophies.extend([
            all_super_rank_trophy(is_all_s_rank),
            multiple_lang_trophy(user_info.language_count),
            long_time_account_trophy(user_info.duration_year),
            ancient_account_trophy(user_info.ancient_account),
            og_account_trophy(user_info.og_account),
            joined_2020_trophy(user_info.joined_2020),
            multiple_organizations_trophy(user_info.total_organizations),
            account_duration_trophy(user_info.duration_days),
        ]);

        Self { trophies }
    }

    pub fn len(&self) -> usize {
        self.trophies.len()
    }

    pub fn items(&self) -> &[Trophy] {
        &self.trophies
    }

    pub fn filter_by_hidden(&mut self) {
        self.trophies
            .retain(|trophy| !trophy.hidden || trophy.rank != Rank::Unknown);
    }

    pub fn filter_by_titles(&mut self, titles: &[String]) {
        let include: HashSet<&str> = titles.iter().map(String::as_str).collect();

        self.trophies.retain(|trophy| {
            trophy
                .filter_titles
                .iter()
                .any(|title| include.contains(*title))
        });
    }

    pub fn filter_by_ranks(&mut self, ranks: &[String]) {
        if ranks.iter().any(|rank| rank.contains('-')) {
            let excluded: HashSet<&str> = ranks
                .iter()
                .filter_map(|rank| rank.strip_prefix('-'))
                .collect();
            self.trophies
                .retain(|trophy| !excluded.contains(trophy.rank.as_str()));
            return;
        }

        let included: HashSet<&str> = ranks.iter().map(String::as_str).collect();
        self.trophies
            .retain(|trophy| included.contains(trophy.rank.as_str()));
    }

    pub fn filter_by_exclusion_titles(&mut self, titles: &[String]) {
        let excluded: HashSet<&str> = titles
            .iter()
            .filter_map(|title| title.strip_prefix('-'))
            .collect();

        if excluded.is_empty() {
            return;
        }

        self.trophies
            .retain(|trophy| !excluded.contains(trophy.title));
    }

    pub fn sort_by_rank(&mut self) {
        self.trophies
            .sort_by_key(|trophy| rank_order_index(trophy.rank));
    }
}

pub fn abridge_score(score: i64) -> String {
    let abs = score.abs();
    if abs < 1 {
        return "0pt".to_string();
    }

    if abs > 999 {
        let signed = if score < 0 {
            -((abs as f64) / 1000.0)
        } else {
            (abs as f64) / 1000.0
        };
        return format!("{signed:.1}kpt");
    }

    format!("{score}pt")
}

fn rank_order_index(rank: Rank) -> usize {
    RANK_ORDER
        .iter()
        .position(|item| *item == rank)
        .unwrap_or(RANK_ORDER.len() - 1)
}

const CONDITION_SECRET_RAINBOW: [RankCondition; 1] = [RankCondition {
    rank: Rank::Secret,
    message: "Rainbow Lang User",
    required_score: 10,
}];

const CONDITION_SECRET_ALL_S: [RankCondition; 1] = [RankCondition {
    rank: Rank::Secret,
    message: "S Rank Hacker",
    required_score: 1,
}];

const CONDITION_SECRET_JOINED_2020: [RankCondition; 1] = [RankCondition {
    rank: Rank::Secret,
    message: "Everything started...",
    required_score: 1,
}];

const CONDITION_SECRET_ANCIENT: [RankCondition; 1] = [RankCondition {
    rank: Rank::Secret,
    message: "Ancient User",
    required_score: 1,
}];

const CONDITION_SECRET_LONG_TIME: [RankCondition; 1] = [RankCondition {
    rank: Rank::Secret,
    message: "Village Elder",
    required_score: 10,
}];

const CONDITION_SECRET_ORG: [RankCondition; 1] = [RankCondition {
    rank: Rank::Secret,
    message: "Jack of all Trades",
    required_score: 3,
}];

const CONDITION_SECRET_OG: [RankCondition; 1] = [RankCondition {
    rank: Rank::Secret,
    message: "OG User",
    required_score: 1,
}];

const CONDITION_REVIEWS: [RankCondition; 8] = [
    RankCondition {
        rank: Rank::Sss,
        message: "God Reviewer",
        required_score: 70,
    },
    RankCondition {
        rank: Rank::Ss,
        message: "Deep Reviewer",
        required_score: 57,
    },
    RankCondition {
        rank: Rank::S,
        message: "Super Reviewer",
        required_score: 45,
    },
    RankCondition {
        rank: Rank::Aaa,
        message: "Ultra Reviewer",
        required_score: 30,
    },
    RankCondition {
        rank: Rank::Aa,
        message: "Hyper Reviewer",
        required_score: 20,
    },
    RankCondition {
        rank: Rank::A,
        message: "Active Reviewer",
        required_score: 8,
    },
    RankCondition {
        rank: Rank::B,
        message: "Intermediate Reviewer",
        required_score: 3,
    },
    RankCondition {
        rank: Rank::C,
        message: "New Reviewer",
        required_score: 1,
    },
];

const CONDITION_ACCOUNT_DURATION: [RankCondition; 8] = [
    RankCondition {
        rank: Rank::Sss,
        message: "Seasoned Veteran",
        required_score: 70,
    },
    RankCondition {
        rank: Rank::Ss,
        message: "Grandmaster",
        required_score: 55,
    },
    RankCondition {
        rank: Rank::S,
        message: "Master Dev",
        required_score: 40,
    },
    RankCondition {
        rank: Rank::Aaa,
        message: "Expert Dev",
        required_score: 28,
    },
    RankCondition {
        rank: Rank::Aa,
        message: "Experienced Dev",
        required_score: 18,
    },
    RankCondition {
        rank: Rank::A,
        message: "Intermediate Dev",
        required_score: 11,
    },
    RankCondition {
        rank: Rank::B,
        message: "Junior Dev",
        required_score: 6,
    },
    RankCondition {
        rank: Rank::C,
        message: "Newbie",
        required_score: 2,
    },
];

const CONDITION_STARS: [RankCondition; 8] = [
    RankCondition {
        rank: Rank::Sss,
        message: "Super Stargazer",
        required_score: 2000,
    },
    RankCondition {
        rank: Rank::Ss,
        message: "High Stargazer",
        required_score: 700,
    },
    RankCondition {
        rank: Rank::S,
        message: "Stargazer",
        required_score: 200,
    },
    RankCondition {
        rank: Rank::Aaa,
        message: "Super Star",
        required_score: 100,
    },
    RankCondition {
        rank: Rank::Aa,
        message: "High Star",
        required_score: 50,
    },
    RankCondition {
        rank: Rank::A,
        message: "You are a Star",
        required_score: 30,
    },
    RankCondition {
        rank: Rank::B,
        message: "Middle Star",
        required_score: 10,
    },
    RankCondition {
        rank: Rank::C,
        message: "First Star",
        required_score: 1,
    },
];

const CONDITION_COMMITS: [RankCondition; 8] = [
    RankCondition {
        rank: Rank::Sss,
        message: "God Committer",
        required_score: 4000,
    },
    RankCondition {
        rank: Rank::Ss,
        message: "Deep Committer",
        required_score: 2000,
    },
    RankCondition {
        rank: Rank::S,
        message: "Super Committer",
        required_score: 1000,
    },
    RankCondition {
        rank: Rank::Aaa,
        message: "Ultra Committer",
        required_score: 500,
    },
    RankCondition {
        rank: Rank::Aa,
        message: "Hyper Committer",
        required_score: 200,
    },
    RankCondition {
        rank: Rank::A,
        message: "High Committer",
        required_score: 100,
    },
    RankCondition {
        rank: Rank::B,
        message: "Middle Committer",
        required_score: 10,
    },
    RankCondition {
        rank: Rank::C,
        message: "First Commit",
        required_score: 1,
    },
];

const CONDITION_FOLLOWERS: [RankCondition; 8] = [
    RankCondition {
        rank: Rank::Sss,
        message: "Super Celebrity",
        required_score: 1000,
    },
    RankCondition {
        rank: Rank::Ss,
        message: "Ultra Celebrity",
        required_score: 400,
    },
    RankCondition {
        rank: Rank::S,
        message: "Hyper Celebrity",
        required_score: 200,
    },
    RankCondition {
        rank: Rank::Aaa,
        message: "Famous User",
        required_score: 100,
    },
    RankCondition {
        rank: Rank::Aa,
        message: "Active User",
        required_score: 50,
    },
    RankCondition {
        rank: Rank::A,
        message: "Dynamic User",
        required_score: 20,
    },
    RankCondition {
        rank: Rank::B,
        message: "Many Friends",
        required_score: 10,
    },
    RankCondition {
        rank: Rank::C,
        message: "First Friend",
        required_score: 1,
    },
];

const CONDITION_ISSUES: [RankCondition; 8] = [
    RankCondition {
        rank: Rank::Sss,
        message: "God Issuer",
        required_score: 1000,
    },
    RankCondition {
        rank: Rank::Ss,
        message: "Deep Issuer",
        required_score: 500,
    },
    RankCondition {
        rank: Rank::S,
        message: "Super Issuer",
        required_score: 200,
    },
    RankCondition {
        rank: Rank::Aaa,
        message: "Ultra Issuer",
        required_score: 100,
    },
    RankCondition {
        rank: Rank::Aa,
        message: "Hyper Issuer",
        required_score: 50,
    },
    RankCondition {
        rank: Rank::A,
        message: "High Issuer",
        required_score: 20,
    },
    RankCondition {
        rank: Rank::B,
        message: "Middle Issuer",
        required_score: 10,
    },
    RankCondition {
        rank: Rank::C,
        message: "First Issue",
        required_score: 1,
    },
];

const CONDITION_PULL_REQUESTS: [RankCondition; 8] = [
    RankCondition {
        rank: Rank::Sss,
        message: "God Puller",
        required_score: 1000,
    },
    RankCondition {
        rank: Rank::Ss,
        message: "Deep Puller",
        required_score: 500,
    },
    RankCondition {
        rank: Rank::S,
        message: "Super Puller",
        required_score: 200,
    },
    RankCondition {
        rank: Rank::Aaa,
        message: "Ultra Puller",
        required_score: 100,
    },
    RankCondition {
        rank: Rank::Aa,
        message: "Hyper Puller",
        required_score: 50,
    },
    RankCondition {
        rank: Rank::A,
        message: "High Puller",
        required_score: 20,
    },
    RankCondition {
        rank: Rank::B,
        message: "Middle Puller",
        required_score: 10,
    },
    RankCondition {
        rank: Rank::C,
        message: "First Pull",
        required_score: 1,
    },
];

const CONDITION_REPOSITORIES: [RankCondition; 8] = [
    RankCondition {
        rank: Rank::Sss,
        message: "God Repo Creator",
        required_score: 50,
    },
    RankCondition {
        rank: Rank::Ss,
        message: "Deep Repo Creator",
        required_score: 45,
    },
    RankCondition {
        rank: Rank::S,
        message: "Super Repo Creator",
        required_score: 40,
    },
    RankCondition {
        rank: Rank::Aaa,
        message: "Ultra Repo Creator",
        required_score: 35,
    },
    RankCondition {
        rank: Rank::Aa,
        message: "Hyper Repo Creator",
        required_score: 30,
    },
    RankCondition {
        rank: Rank::A,
        message: "High Repo Creator",
        required_score: 20,
    },
    RankCondition {
        rank: Rank::B,
        message: "Middle Repo Creator",
        required_score: 10,
    },
    RankCondition {
        rank: Rank::C,
        message: "First Repository",
        required_score: 1,
    },
];

fn multiple_lang_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "MultiLanguage",
        &["MultipleLang", "MultiLanguage"],
        true,
        &CONDITION_SECRET_RAINBOW,
        None,
    )
}

fn all_super_rank_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "AllSuperRank",
        &["AllSuperRank"],
        true,
        &CONDITION_SECRET_ALL_S,
        Some("All S Rank"),
    )
}

fn joined_2020_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Joined2020",
        &["Joined2020"],
        true,
        &CONDITION_SECRET_JOINED_2020,
        Some("Joined 2020"),
    )
}

fn ancient_account_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "AncientUser",
        &["AncientUser"],
        true,
        &CONDITION_SECRET_ANCIENT,
        Some("Before 2010"),
    )
}

fn long_time_account_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "LongTimeUser",
        &["LongTimeUser"],
        true,
        &CONDITION_SECRET_LONG_TIME,
        None,
    )
}

fn multiple_organizations_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Organizations",
        &["Organizations", "Orgs", "Teams"],
        true,
        &CONDITION_SECRET_ORG,
        None,
    )
}

fn og_account_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "OGUser",
        &["OGUser"],
        true,
        &CONDITION_SECRET_OG,
        Some("Joined 2008"),
    )
}

fn total_reviews_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Reviews",
        &["Review", "Reviews"],
        false,
        &CONDITION_REVIEWS,
        None,
    )
}

fn account_duration_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Experience",
        &["Experience", "Duration", "Since"],
        false,
        &CONDITION_ACCOUNT_DURATION,
        None,
    )
}

fn total_star_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Stars",
        &["Star", "Stars"],
        false,
        &CONDITION_STARS,
        None,
    )
}

fn total_commit_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Commits",
        &["Commit", "Commits"],
        false,
        &CONDITION_COMMITS,
        None,
    )
}

fn total_follower_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Followers",
        &["Follower", "Followers"],
        false,
        &CONDITION_FOLLOWERS,
        None,
    )
}

fn total_issue_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Issues",
        &["Issue", "Issues"],
        false,
        &CONDITION_ISSUES,
        None,
    )
}

fn total_pull_request_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "PullRequest",
        &["PR", "PullRequest", "Pulls", "Puller"],
        false,
        &CONDITION_PULL_REQUESTS,
        None,
    )
}

fn total_repository_trophy(score: i64) -> Trophy {
    Trophy::new(
        score,
        "Repositories",
        &["Repo", "Repository", "Repositories"],
        false,
        &CONDITION_REPOSITORIES,
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::abridge_score;

    #[test]
    fn abridge_score_formats_as_expected() {
        assert_eq!(abridge_score(0), "0pt");
        assert_eq!(abridge_score(5), "5pt");
        assert_eq!(abridge_score(1000), "1.0kpt");
    }
}
