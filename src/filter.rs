use std::fmt;

/// Represents a component of a FogBugz search query.
#[derive(Clone, Debug)]
enum SearchComponent {
    /// A single search term, e.g., `apple`.
    Term(String),
    /// A search phrase, e.g., `"apple peach"`.
    Phrase(String),
    /// A negated term, e.g., `-peach`.
    NegatedTerm(String),
    /// An axis search, e.g., `project:Widget`.
    Axis { axis: String, query: String },
    /// A negated axis search, e.g., `-title:pear`.
    NegatedAxis { axis: String, query: String },
    /// An exact axis search using `:=`, e.g., `project:=1`.
    ExactAxis { axis: String, query: String },
    /// A group of components joined by OR, e.g., `(assignedto:"A" OR assignedto:"B")`.
    Or(Vec<SearchComponent>),
}

impl SearchComponent {
    /// Converts the search component into its string representation for the final query.
    fn stringify(&self) -> String {
        match self {
            SearchComponent::Term(term) => term.clone(),
            SearchComponent::Phrase(phrase) => format!("\"{}\"", phrase.replace("\"", "\\\"")),
            SearchComponent::NegatedTerm(term) => format!("-{}", term),
            SearchComponent::Axis { axis, query }
            | SearchComponent::NegatedAxis { axis, query } => {
                // Quote query if it contains spaces, colons, quotes, or a wildcard (but not just '*' itself).
                // Also escape any internal quotes.
                let needs_quoting = query.contains(' ')
                    || query.contains(':')
                    || query.contains('"')
                    || query.contains("..")
                    || query.starts_with('-'); // Quote date ranges and descending order

                let formatted_query =
                    if needs_quoting && !(query.starts_with('"') && query.ends_with('"')) {
                        // Add quotes if needed and not already present (e.g., for OrderBy descending)
                        format!("\"{}\"", query.replace("\"", "\\\""))
                    } else {
                        // Still escape internal quotes even if not adding surrounding quotes
                        query.replace("\"", "\\\"")
                    };

                match self {
                    SearchComponent::Axis { .. } => format!("{}:{}", axis, formatted_query),
                    SearchComponent::NegatedAxis { .. } => format!("-{}:{}", axis, formatted_query),
                    _ => unreachable!(), // Should not happen due to outer match pattern
                }
            }
            SearchComponent::ExactAxis { axis, query } => {
                // Exact axis query (like an ID) usually doesn't need quoting, but escape internal quotes just in case.
                let formatted_query = query.replace("\"", "\\\"");
                format!("{}:={}", axis, formatted_query)
            }
            SearchComponent::Or(components) => {
                // Filter out potential empty components before joining
                let parts: Vec<String> = components
                    .iter()
                    .map(|c| c.stringify())
                    .filter(|s| !s.is_empty())
                    .collect();

                if parts.is_empty() {
                    String::new() // Return empty string if OR group becomes empty
                } else {
                    format!("({})", parts.join(" OR ")) // Wrap OR group in parentheses
                }
            }
        }
    }
}

/// Helper struct for building the components within an OR group.
#[derive(Debug, Default)]
pub struct OrBuilder {
    components: Vec<SearchComponent>,
}

impl OrBuilder {
    /// Creates a new, empty OR builder. Typically used internally.
    fn new() -> Self {
        Default::default()
    }

    /// Adds a simple term to the OR group.
    pub fn term(mut self, term: &str) -> Self {
        if !term.trim().is_empty() {
            self.components
                .push(SearchComponent::Term(term.to_string()));
        }
        self
    }

    /// Adds a phrase to the OR group.
    pub fn phrase(mut self, phrase: &str) -> Self {
        if !phrase.trim().is_empty() {
            self.components
                .push(SearchComponent::Phrase(phrase.to_string()));
        }
        self
    }

    /// Adds an axis search to the OR group.
    pub fn axis(mut self, axis: &str, query: &str) -> Self {
        if !axis.trim().is_empty() && !query.trim().is_empty() {
            self.components.push(SearchComponent::Axis {
                axis: axis.to_string(),
                query: query.to_string(),
            });
        }
        self
    }

    // --- Add common axis helpers specific to OR groups if desired ---

    /// Adds an `assignedto` axis search to the OR group.
    pub fn assigned_to(self, user_name: &str) -> Self {
        self.axis("assignedto", user_name)
    }

    /// Adds a `resolvedby` axis search to the OR group.
    pub fn resolved_by(self, user_name: &str) -> Self {
        self.axis("resolvedby", user_name)
    }

    /// Adds an `editedby` axis search to the OR group.
    pub fn edited_by(self, user_name: &str) -> Self {
        self.axis("editedby", user_name)
    }
}

/// Builds a FogBugz search query string by combining various filters.
/// Filters added are implicitly joined by AND, unless grouped using `or()`.
#[derive(Debug, Default)]
pub struct FogBugzSearchBuilder {
    components: Vec<SearchComponent>,
}

impl FogBugzSearchBuilder {
    /// Creates a new, empty search builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a simple search term (implicitly ANDed with previous components).
    /// Example: `term("apple")` adds `apple`.
    pub fn term(mut self, term: &str) -> Self {
        if !term.trim().is_empty() {
            self.components
                .push(SearchComponent::Term(term.to_string()));
        }
        self
    }

    /// Adds a search phrase (implicitly ANDed with previous components).
    /// Handles necessary quoting and escaping.
    /// Example: `phrase("apple peach")` adds `"apple peach"`.
    pub fn phrase(mut self, phrase: &str) -> Self {
        if !phrase.trim().is_empty() {
            self.components
                .push(SearchComponent::Phrase(phrase.to_string()));
        }
        self
    }

    /// Adds a negated term (implicitly ANDed with previous components).
    /// Example: `negated_term("peach")` adds `-peach`.
    pub fn negated_term(mut self, term: &str) -> Self {
        if !term.trim().is_empty() {
            // Basic validation: ensure term doesn't start with '-' already
            let clean_term = term.trim().trim_start_matches('-').to_string();
            if !clean_term.is_empty() {
                self.components
                    .push(SearchComponent::NegatedTerm(clean_term));
            }
        }
        self
    }

    /// Adds an axis search (implicitly ANDed with previous components).
    /// Handles necessary quoting and escaping for the query value.
    /// Example: `axis("project", "Widget Factory")` adds `project:"Widget Factory"`.
    pub fn axis(mut self, axis: &str, query: &str) -> Self {
        let axis_trimmed = axis.trim();
        let query_trimmed = query.trim();
        if !axis_trimmed.is_empty() && !query_trimmed.is_empty() {
            self.components.push(SearchComponent::Axis {
                axis: axis_trimmed.to_string(),
                query: query_trimmed.to_string(),
            });
        }
        self
    }

    /// Adds a negated axis search (implicitly ANDed with previous components).
    /// Handles necessary quoting and escaping for the query value.
    /// Example: `negated_axis("title", "Review")` adds `-title:Review`.
    pub fn negated_axis(mut self, axis: &str, query: &str) -> Self {
        let axis_trimmed = axis.trim().trim_start_matches('-'); // Remove potential leading '-' from axis name
        let query_trimmed = query.trim();
        if !axis_trimmed.is_empty() && !query_trimmed.is_empty() {
            self.components.push(SearchComponent::NegatedAxis {
                axis: axis_trimmed.to_string(),
                query: query_trimmed.to_string(),
            });
        }
        self
    }

    /// Adds an exact axis search using `:=` (implicitly ANDed with previous components).
    /// Useful for searching by ID, e.g., project ID.
    /// Example: `exact_axis("project", "1")` adds `project:=1`.
    pub fn exact_axis(mut self, axis: &str, query: &str) -> Self {
        let axis_trimmed = axis.trim();
        let query_trimmed = query.trim();
        if !axis_trimmed.is_empty() && !query_trimmed.is_empty() {
            self.components.push(SearchComponent::ExactAxis {
                axis: axis_trimmed.to_string(),
                query: query_trimmed.to_string(),
            });
        }
        self
    }

    /// Adds a group of filters joined by OR (implicitly ANDed with previous components).
    /// Use the provided `OrBuilder` in the closure to define the OR conditions.
    /// Example: `or(|group| group.assigned_to("Tester 1").assigned_to("Tester 2"))`
    /// adds `(assignedto:"Tester 1" OR assignedto:"Tester 2")`.
    pub fn or(mut self, build_or_group: impl FnOnce(OrBuilder) -> OrBuilder) -> Self {
        let builder = OrBuilder::new();
        let finished_builder = build_or_group(builder);
        // Add the OR group only if it contains components
        if !finished_builder.components.is_empty() {
            self.components
                .push(SearchComponent::Or(finished_builder.components));
        }
        self
    }

    // --- Common Axis Shortcuts ---

    /// Adds `project:<project_name>` axis search.
    pub fn project(self, project_name: &str) -> Self {
        self.axis("project", project_name)
    }

    /// Adds `project:=<project_id>` axis search for exact match by ID.
    pub fn project_id(self, project_id: u32) -> Self {
        self.exact_axis("project", &project_id.to_string())
    }

    /// Adds `assignedto:<user_name>` axis search.
    pub fn assigned_to(self, user_name: &str) -> Self {
        self.axis("assignedto", user_name)
    }

    /// Adds `openedby:<user_name>` axis search.
    pub fn opened_by(self, user_name: &str) -> Self {
        self.axis("openedby", user_name)
    }

    /// Adds `editedby:<user_name>` axis search.
    /// Combine with `also_edited_by` for multiple editors.
    pub fn edited_by(self, user_name: &str) -> Self {
        self.axis("editedby", user_name)
    }

    /// Adds `alsoeditedby:<user_name>` axis search.
    /// Should be used in conjunction with `edited_by`.
    pub fn also_edited_by(self, user_name: &str) -> Self {
        self.axis("alsoeditedby", user_name)
    }

    /// Adds `resolvedby:<user_name>` axis search.
    pub fn resolved_by(self, user_name: &str) -> Self {
        self.axis("resolvedby", user_name)
    }

    /// Adds `status:<status_name>` axis search.
    pub fn status(self, status_name: &str) -> Self {
        self.axis("status", status_name)
    }

    /// Adds `tag:<tag_name>` axis search (exact match by default in FogBugz).
    pub fn tag(self, tag_name: &str) -> Self {
        self.axis("tag", tag_name)
    }

    /// Adds a wildcard tag search, e.g., `tag:"mo*"`.
    /// Handles necessary quoting for the wildcard pattern.
    pub fn tag_wildcard(self, tag_pattern: &str) -> Self {
        // Ensure the pattern actually contains a wildcard or needs it
        let query = if tag_pattern.ends_with('*') {
            tag_pattern.to_string()
        } else {
            format!("{}*", tag_pattern) // Append wildcard if missing
        };
        self.axis("tag", &query)
    }

    /// Adds `type:<doc_type>` axis search ("case", "wiki", "discuss").
    pub fn type_is(self, doc_type: &str) -> Self {
        self.axis("type", doc_type)
    }

    /// Adds `ixBug:<case_number>` axis search.
    pub fn case_number(self, case_number: u32) -> Self {
        self.axis("ixBug", &case_number.to_string()) // ixBug uses standard axis syntax
    }

    // --- Date Axis Shortcuts ---
    // These accept string representations as FogBugz date handling is flexible.
    // Examples: "today", "yesterday", "March 2007", "3/26/2007..6/8/2007", "-3w..-1w"

    /// Adds `edited:<date_query>` axis search.
    pub fn edited_date(self, date_query: &str) -> Self {
        self.axis("edited", date_query)
    }

    /// Adds `opened:<date_query>` axis search.
    pub fn opened_date(self, date_query: &str) -> Self {
        self.axis("opened", date_query)
    }

    /// Adds `resolved:<date_query>` axis search.
    pub fn resolved_date(self, date_query: &str) -> Self {
        self.axis("resolved", date_query)
    }

    /// Adds `closed:<date_query>` axis search.
    pub fn closed_date(self, date_query: &str) -> Self {
        self.axis("closed", date_query)
    }

    /// Adds `due:<date_query>` axis search.
    pub fn due_date(self, date_query: &str) -> Self {
        self.axis("due", date_query)
    }

    // --- Wildcard / Existence Axis Shortcuts ---

    /// Adds search for items *having* a value for the specified axis.
    /// Example: `has_axis("tag")` adds `tag:*`.
    pub fn has_axis(self, axis: &str) -> Self {
        self.axis(axis, "*")
    }

    /// Adds search for items *missing* a value for the specified axis.
    /// Example: `missing_axis("tag")` adds `-tag:*`.
    pub fn missing_axis(self, axis: &str) -> Self {
        self.negated_axis(axis, "*")
    }

    // --- Ordering ---

    /// Adds an `OrderBy:<axis>` component to sort results.
    /// Call multiple times for secondary sort orders.
    /// `descending = true` adds `OrderBy:"-<axis>"`.
    pub fn order_by(mut self, axis: &str, descending: bool) -> Self {
        let axis_trimmed = axis.trim();
        if !axis_trimmed.is_empty() {
            let order_axis_query = if descending {
                format!("-{}", axis_trimmed)
            } else {
                axis_trimmed.to_string()
            };
            self.components.push(SearchComponent::Axis {
                axis: "OrderBy".to_string(),
                query: order_axis_query,
            });
        }
        self
    }

    // --- Finalization ---

    /// Builds the final FogBugz search query string.
    /// Joins all components with spaces (implicit AND).
    pub fn build(self) -> String {
        let parts: Vec<String> = self
            .components
            .iter()
            .map(|c| c.stringify())
            .filter(|s| !s.is_empty()) // Filter out empty strings (e.g., from empty OR groups)
            .collect();
        parts.join(" ")
    }
}

// Allow the builder itself to be displayed as the built string (for convenience)
impl fmt::Display for FogBugzSearchBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Note: This clones the components to build the string representation.
        // Avoid calling this repeatedly in performance-sensitive code if the builder is large.
        let parts: Vec<String> = self
            .components
            .iter()
            .map(|c| c.stringify())
            .filter(|s| !s.is_empty())
            .collect();
        write!(f, "{}", parts.join(" "))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_search_terms() {
        // Single term
        let query = FogBugzSearchBuilder::new().term("apple").build();
        assert_eq!(query, "apple");

        // Multiple terms (AND)
        let query = FogBugzSearchBuilder::new()
            .term("apple")
            .term("peach")
            .build();
        assert_eq!(query, "apple peach");

        // Phrase
        let query = FogBugzSearchBuilder::new().phrase("apple peach").build();
        assert_eq!(query, "\"apple peach\"");

        // Mix of terms and phrase
        let query = FogBugzSearchBuilder::new()
            .term("banana")
            .phrase("apple peach")
            .build();
        assert_eq!(query, "banana \"apple peach\"");
    }

    #[test]
    fn test_negated_terms() {
        // Negated term
        let query = FogBugzSearchBuilder::new().negated_term("peach").build();
        assert_eq!(query, "-peach");

        // Mix with regular term
        let query = FogBugzSearchBuilder::new()
            .term("apple")
            .negated_term("peach")
            .build();
        assert_eq!(query, "apple -peach");
    }

    #[test]
    fn test_axis_search() {
        // Simple axis
        let query = FogBugzSearchBuilder::new()
            .axis("project", "Widget")
            .build();
        assert_eq!(query, "project:Widget");

        // Axis with spaces (should be quoted)
        let query = FogBugzSearchBuilder::new()
            .axis("project", "Widget Factory")
            .build();
        assert_eq!(query, "project:\"Widget Factory\"");

        // Exact axis
        let query = FogBugzSearchBuilder::new()
            .exact_axis("project", "1")
            .build();
        assert_eq!(query, "project:=1");

        // Negated axis
        let query = FogBugzSearchBuilder::new()
            .negated_axis("title", "pear")
            .build();
        assert_eq!(query, "-title:pear");
    }

    #[test]
    fn test_quote_escaping() {
        // Quotes in phrases
        let query = FogBugzSearchBuilder::new()
            .phrase("apple \"red\" peach")
            .build();
        assert_eq!(query, "\"apple \\\"red\\\" peach\"");

        // Quotes in axis values
        let query = FogBugzSearchBuilder::new()
            .axis("openedby", "Joel \"The Bossman\" Spolsky")
            .build();
        assert_eq!(query, "openedby:\"Joel \\\"The Bossman\\\" Spolsky\"");
    }

    #[test]
    fn test_or_groups() {
        // Simple OR
        let query = FogBugzSearchBuilder::new()
            .or(|or| or.term("apple").term("peach"))
            .build();
        assert_eq!(query, "(apple OR peach)");

        // OR with axis searches
        let query = FogBugzSearchBuilder::new()
            .or(|or| or.assigned_to("Tester 1").assigned_to("Tester 2"))
            .build();
        assert_eq!(
            query,
            "(assignedto:\"Tester 1\" OR assignedto:\"Tester 2\")"
        );

        // Complex query with OR groups
        let query = FogBugzSearchBuilder::new()
            .term("newfeature")
            .or(|or| or.assigned_to("Tester 1").assigned_to("Tester 2"))
            .or(|or| or.resolved_by("Developer1").resolved_by("Developer2"))
            .build();
        assert_eq!(
            query,
            "newfeature (assignedto:\"Tester 1\" OR assignedto:\"Tester 2\") (resolvedby:Developer1 OR resolvedby:Developer2)"
        );
    }

    #[test]
    fn test_date_searches() {
        // Simple date search
        let query = FogBugzSearchBuilder::new().edited_date("today").build();
        assert_eq!(query, "edited:today");

        // Date range
        let query = FogBugzSearchBuilder::new()
            .resolved_date("3/26/2007..6/8/2007")
            .build();
        assert_eq!(query, "resolved:\"3/26/2007..6/8/2007\"");

        // Relative date
        let query = FogBugzSearchBuilder::new().due_date("-1d..").build();
        assert_eq!(query, "due:\"-1d..\"");
    }

    #[test]
    fn test_wildcard_searches() {
        // Has tag
        let query = FogBugzSearchBuilder::new().has_axis("tag").build();
        assert_eq!(query, "tag:*");

        // Missing due date
        let query = FogBugzSearchBuilder::new().missing_axis("due").build();
        assert_eq!(query, "-due:*");

        // Tag wildcard search
        let query = FogBugzSearchBuilder::new().tag_wildcard("mo").build();
        assert_eq!(query, "tag:mo*");
    }

    #[test]
    fn test_order_by() {
        // Ascending order
        let query = FogBugzSearchBuilder::new()
            .order_by("Milestone", false)
            .build();
        assert_eq!(query, "OrderBy:Milestone");

        // Descending order
        let query = FogBugzSearchBuilder::new()
            .order_by("Milestone", true)
            .build();
        assert_eq!(query, "OrderBy:\"-Milestone\"");

        // Multiple sort orders
        let query = FogBugzSearchBuilder::new()
            .order_by("Milestone", false)
            .order_by("Priority", false)
            .build();
        assert_eq!(query, "OrderBy:Milestone OrderBy:Priority");
    }

    #[test]
    fn test_complex_query() {
        let query = FogBugzSearchBuilder::new()
            .project("Sample Project")
            .status("Active")
            .or(|or| or.assigned_to("Alice").assigned_to("Bob"))
            .edited_date("-1w..today")
            .negated_axis("tag", "obsolete")
            .order_by("Priority", false)
            .order_by("Due", true)
            .build();

        assert_eq!(
            query,
            "project:\"Sample Project\" status:Active (assignedto:Alice OR assignedto:Bob) edited:\"-1w..today\" -tag:obsolete OrderBy:Priority OrderBy:\"-Due\""
        );
    }
}
