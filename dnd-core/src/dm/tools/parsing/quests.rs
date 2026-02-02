//! Quest tool parsing - converts quest tool calls into game Intents.

use crate::rules::Intent;
use serde_json::Value;

/// Parse quest-related tool calls into Intents.
pub fn parse_quests_tool(name: &str, input: &Value) -> Option<Intent> {
    match name {
        "create_quest" => {
            let quest_name = input.get("name")?.as_str()?.to_string();
            let description = input.get("description")?.as_str()?.to_string();
            let giver = input
                .get("giver")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Parse objectives
            let objectives = input
                .get("objectives")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|obj| {
                            let desc = obj.get("description")?.as_str()?.to_string();
                            let optional = obj
                                .get("optional")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            Some((desc, optional))
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Parse rewards
            let rewards = input
                .get("rewards")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|r| r.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(Intent::CreateQuest {
                name: quest_name,
                description,
                giver,
                objectives,
                rewards,
            })
        }

        "add_quest_objective" => {
            let quest_name = input.get("quest_name")?.as_str()?.to_string();
            let objective = input.get("objective")?.as_str()?.to_string();
            let optional = input
                .get("optional")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            Some(Intent::AddQuestObjective {
                quest_name,
                objective,
                optional,
            })
        }

        "complete_objective" => {
            let quest_name = input.get("quest_name")?.as_str()?.to_string();
            let objective_description = input.get("objective_description")?.as_str()?.to_string();

            Some(Intent::CompleteObjective {
                quest_name,
                objective_description,
            })
        }

        "complete_quest" => {
            let quest_name = input.get("quest_name")?.as_str()?.to_string();
            let completion_note = input
                .get("completion_note")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(Intent::CompleteQuest {
                quest_name,
                completion_note,
            })
        }

        "fail_quest" => {
            let quest_name = input.get("quest_name")?.as_str()?.to_string();
            let failure_reason = input.get("failure_reason")?.as_str()?.to_string();

            Some(Intent::FailQuest {
                quest_name,
                failure_reason,
            })
        }

        "update_quest" => {
            let quest_name = input.get("quest_name")?.as_str()?.to_string();
            let new_description = input
                .get("new_description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let add_rewards = input
                .get("add_rewards")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|r| r.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            Some(Intent::UpdateQuest {
                quest_name,
                new_description,
                add_rewards,
            })
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_quest() {
        let input = json!({
            "name": "The Missing Merchant",
            "description": "Find the merchant who disappeared on the road to Riverdale.",
            "giver": "Mayor Thornwood",
            "objectives": [
                {"description": "Investigate the merchant's last known location", "optional": false},
                {"description": "Find clues about what happened", "optional": false},
                {"description": "Rescue the merchant or recover his goods", "optional": false}
            ],
            "rewards": ["100 gold", "Reputation with Thornwood"]
        });

        let intent = parse_quests_tool("create_quest", &input);
        assert!(intent.is_some());

        if let Some(Intent::CreateQuest {
            name,
            giver,
            objectives,
            rewards,
            ..
        }) = intent
        {
            assert_eq!(name, "The Missing Merchant");
            assert_eq!(giver, Some("Mayor Thornwood".to_string()));
            assert_eq!(objectives.len(), 3);
            assert_eq!(rewards.len(), 2);
        } else {
            panic!("Expected CreateQuest intent");
        }
    }

    #[test]
    fn test_complete_objective() {
        let input = json!({
            "quest_name": "The Missing Merchant",
            "objective_description": "Find clues"
        });

        let intent = parse_quests_tool("complete_objective", &input);
        assert!(intent.is_some());

        if let Some(Intent::CompleteObjective {
            quest_name,
            objective_description,
        }) = intent
        {
            assert_eq!(quest_name, "The Missing Merchant");
            assert_eq!(objective_description, "Find clues");
        } else {
            panic!("Expected CompleteObjective intent");
        }
    }

    #[test]
    fn test_fail_quest() {
        let input = json!({
            "quest_name": "Save the Princess",
            "failure_reason": "The dragon ate her while you were shopping"
        });

        let intent = parse_quests_tool("fail_quest", &input);
        assert!(intent.is_some());

        if let Some(Intent::FailQuest {
            quest_name,
            failure_reason,
        }) = intent
        {
            assert_eq!(quest_name, "Save the Princess");
            assert!(failure_reason.contains("dragon"));
        } else {
            panic!("Expected FailQuest intent");
        }
    }
}
