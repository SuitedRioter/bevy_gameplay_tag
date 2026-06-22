use crate::gameplay_tag_container::{
    GameplayTagContainer, GameplayTagQuery, GameplayTagQueryExpression,
};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct GameplayTagRequirements {
    require_tags: GameplayTagContainer,
    ignore_tags: GameplayTagContainer,
    tag_query: GameplayTagQuery,
}

impl GameplayTagRequirements {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_parts(
        require: GameplayTagContainer,
        ignore: GameplayTagContainer,
        query: GameplayTagQuery,
    ) -> Self {
        GameplayTagRequirements {
            require_tags: require,
            ignore_tags: ignore,
            tag_query: query,
        }
    }

    pub fn require_tags(&self) -> &GameplayTagContainer {
        &self.require_tags
    }

    pub fn require_tags_mut(&mut self) -> &mut GameplayTagContainer {
        &mut self.require_tags
    }

    pub fn ignore_tags(&self) -> &GameplayTagContainer {
        &self.ignore_tags
    }

    pub fn ignore_tags_mut(&mut self) -> &mut GameplayTagContainer {
        &mut self.ignore_tags
    }

    pub fn query(&self) -> &GameplayTagQuery {
        &self.tag_query
    }

    pub fn with_query(mut self, query: GameplayTagQuery) -> Self {
        self.tag_query = query;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.require_tags.is_empty() && self.ignore_tags.is_empty() && self.tag_query.is_empty()
    }

    pub fn requirements_met(&self, container_to_check: &GameplayTagContainer) -> bool {
        let has_require_met = container_to_check.has_all(&self.require_tags);
        let has_ignore_met = container_to_check.has_any(&self.ignore_tags);
        let has_query_met = self.tag_query.is_empty() || self.tag_query.matches(container_to_check);
        has_require_met && !has_ignore_met && has_query_met
    }

    pub fn matches(&self, container_to_check: &GameplayTagContainer) -> bool {
        self.requirements_met(container_to_check)
    }

    pub fn convert_tag_fields_to_tag_query(&self) -> GameplayTagQuery {
        let has_require = !self.require_tags.is_empty();
        let has_ignore = !self.ignore_tags.is_empty();
        if !has_ignore && !has_require {
            return GameplayTagQuery::new();
        }
        let mut requirements_expression = GameplayTagQueryExpression::new();
        let mut ignore_expression = GameplayTagQueryExpression::new();
        let mut root_expression = GameplayTagQueryExpression::new();
        if has_require && has_ignore {
            requirements_expression
                .all_tags_match()
                .add_tags(&self.require_tags);
            ignore_expression
                .no_tags_match()
                .add_tags(&self.ignore_tags);
            root_expression
                .all_expr_match()
                .add_expr(requirements_expression)
                .add_expr(ignore_expression);
        } else if has_require {
            requirements_expression
                .all_tags_match()
                .add_tags(&self.require_tags);
            root_expression
                .all_expr_match()
                .add_expr(requirements_expression);
        } else {
            ignore_expression
                .no_tags_match()
                .add_tags(&self.ignore_tags);
            root_expression.all_expr_match().add_expr(ignore_expression);
        }
        let mut query = GameplayTagQuery::new();
        query.build(root_expression);
        query
    }

    pub fn to_query(&self) -> GameplayTagQuery {
        self.convert_tag_fields_to_tag_query()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameplayTag, GameplayTagsManager, GameplayTagsSettings};
    use bevy::prelude::World;

    fn test_tags_manager() -> GameplayTagsManager {
        let mut world = World::new();
        world.insert_resource(GameplayTagsSettings {
            json_data: r#"
            [
                { "tag_name": "Ability.Skill.Fire", "description": "Fire skill" },
                { "tag_name": "Status.Buff.Haste", "description": "Haste buff" },
                { "tag_name": "Status.Debuff.Silence", "description": "Silence debuff" }
            ]
            "#
            .to_string(),
            data_path: None,
        });
        world.init_resource::<GameplayTagsManager>();
        world.remove_resource::<GameplayTagsManager>().unwrap()
    }

    fn container_with_tags(tags_manager: &GameplayTagsManager, tags: &[&str]) -> GameplayTagContainer {
        let mut container = GameplayTagContainer::new();
        for tag in tags {
            container.add_tag(GameplayTag::new(tag), tags_manager);
        }
        container
    }

    #[test]
    fn is_empty_only_when_all_sources_are_empty() {
        let tags_manager = test_tags_manager();
        let empty = GameplayTagRequirements::from_parts(
            GameplayTagContainer::new(),
            GameplayTagContainer::new(),
            GameplayTagQuery::new(),
        );
        assert!(empty.is_empty());

        let require_only = GameplayTagRequirements::from_parts(
            container_with_tags(&tags_manager, &["Ability.Skill.Fire"]),
            GameplayTagContainer::new(),
            GameplayTagQuery::new(),
        );
        assert!(!require_only.is_empty());

        let ignore_only = GameplayTagRequirements::from_parts(
            GameplayTagContainer::new(),
            container_with_tags(&tags_manager, &["Status.Debuff.Silence"]),
            GameplayTagQuery::new(),
        );
        assert!(!ignore_only.is_empty());
    }

    #[test]
    fn requirements_met_handles_require_and_ignore_tags() {
        let tags_manager = test_tags_manager();
        let requirements = GameplayTagRequirements::from_parts(
            container_with_tags(&tags_manager, &["Ability.Skill.Fire"]),
            container_with_tags(&tags_manager, &["Status.Debuff.Silence"]),
            GameplayTagQuery::new(),
        );

        let allowed = container_with_tags(&tags_manager, &["Ability.Skill.Fire", "Status.Buff.Haste"]);
        let missing_required = container_with_tags(&tags_manager, &["Status.Buff.Haste"]);
        let blocked = container_with_tags(
            &tags_manager,
            &["Ability.Skill.Fire", "Status.Debuff.Silence"],
        );

        assert!(requirements.requirements_met(&allowed));
        assert!(!requirements.requirements_met(&missing_required));
        assert!(!requirements.requirements_met(&blocked));
    }

    #[test]
    fn convert_tag_fields_to_tag_query_matches_requirements_semantics() {
        let tags_manager = test_tags_manager();
        let require_only = GameplayTagRequirements::from_parts(
            container_with_tags(&tags_manager, &["Ability.Skill.Fire"]),
            GameplayTagContainer::new(),
            GameplayTagQuery::new(),
        );
        let require_query = require_only.convert_tag_fields_to_tag_query();
        let matching = container_with_tags(&tags_manager, &["Ability.Skill.Fire"]);
        let missing = container_with_tags(&tags_manager, &["Status.Buff.Haste"]);
        assert!(require_query.matches(&matching));
        assert!(!require_query.matches(&missing));

        let ignore_only = GameplayTagRequirements::from_parts(
            GameplayTagContainer::new(),
            container_with_tags(&tags_manager, &["Status.Debuff.Silence"]),
            GameplayTagQuery::new(),
        );
        let ignore_query = ignore_only.convert_tag_fields_to_tag_query();
        assert!(ignore_query.matches(&matching));
        let silenced = container_with_tags(&tags_manager, &["Status.Debuff.Silence"]);
        assert!(!ignore_query.matches(&silenced));
    }

    #[test]
    fn alias_methods_match_existing_requirements_behavior() {
        let tags_manager = test_tags_manager();
        let mut requirements = GameplayTagRequirements::new();
        requirements
            .require_tags_mut()
            .add_tag(GameplayTag::new("Ability.Skill.Fire"), &tags_manager);
        requirements
            .ignore_tags_mut()
            .add_tag(GameplayTag::new("Status.Debuff.Silence"), &tags_manager);

        let allowed = container_with_tags(&tags_manager, &["Ability.Skill.Fire"]);
        let blocked = container_with_tags(
            &tags_manager,
            &["Ability.Skill.Fire", "Status.Debuff.Silence"],
        );

        assert_eq!(requirements.matches(&allowed), requirements.requirements_met(&allowed));
        assert_eq!(requirements.matches(&blocked), requirements.requirements_met(&blocked));
        assert_eq!(requirements.to_query(), requirements.convert_tag_fields_to_tag_query());
        assert_eq!(requirements.require_tags(), &container_with_tags(&tags_manager, &["Ability.Skill.Fire"]));
        assert_eq!(requirements.ignore_tags(), &container_with_tags(&tags_manager, &["Status.Debuff.Silence"]));
        assert!(requirements.query().is_empty());
    }
}
