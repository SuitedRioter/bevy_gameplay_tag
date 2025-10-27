use crate::gameplay_tag_container::{GameplayTagContainer, GameplayTagQuery, GameplayTagQueryExpression};

#[derive(Debug)]
pub struct GameplayTagRequirements {
    require_tags: GameplayTagContainer,
    ignore_tags: GameplayTagContainer,
    tag_query: GameplayTagQuery,
}

impl Default for GameplayTagRequirements {
    fn default() -> Self {
        GameplayTagRequirements {
            require_tags: GameplayTagContainer::new(),
            ignore_tags: GameplayTagContainer::new(),
            tag_query: GameplayTagQuery::new(),
        }
    }
}

impl GameplayTagRequirements {
    pub fn new(
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

    pub fn is_empty(&self) -> bool {
        let has_require = !self.require_tags.is_empty();
        let has_ignore = !self.ignore_tags.is_empty();
        let has_query = !self.tag_query.is_empty();

        has_require && has_ignore && has_query
    }

    pub fn requirements_met(&self, container_to_check: &GameplayTagContainer) -> bool {
        let has_require_met = container_to_check.has_all(&self.require_tags);
        let has_ignore_met = container_to_check.has_any(&self.ignore_tags);
        let has_query_met = self.tag_query.is_empty() || self.tag_query.matches(container_to_check);
        if has_require_met && !has_ignore_met && has_query_met {
            true
        } else {
            false
        }
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
            ignore_expression
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
}