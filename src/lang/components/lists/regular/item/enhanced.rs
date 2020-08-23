use super::{ListItem, ListItemContent};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents additional attributes that can be added to an enhanced list item
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum EnhancedListItemAttribute {
    /// Flags list item as a TODO item that has not been completed
    TodoIncomplete,

    /// Flags list item as a TODO item that is partially complete (1-33%)
    TodoPartiallyComplete1,

    /// Flags list item as a TODO item that is partially complete (34-66%)
    TodoPartiallyComplete2,

    /// Flags list item as a TODO item that is partially complete (67-99%)
    TodoPartiallyComplete3,

    /// Flags list item as a TODO item that is complete
    TodoComplete,

    /// Flags list item as a TODO item that has been rejected
    TodoRejected,
}

impl EnhancedListItemAttribute {
    pub fn todo_attributes() -> Vec<Self> {
        vec![
            Self::TodoIncomplete,
            Self::TodoPartiallyComplete1,
            Self::TodoPartiallyComplete2,
            Self::TodoPartiallyComplete3,
            Self::TodoComplete,
            Self::TodoRejected,
        ]
    }
}

/// Represents a wrapper around a basic list item to provide extra information
/// and functionality without needing to implement it for ordered/unordered
/// list item instances directly
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct EnhancedListItem {
    pub item: ListItem,
    pub attributes: HashSet<EnhancedListItemAttribute>,
}

impl From<ListItem> for EnhancedListItem {
    fn from(item: ListItem) -> Self {
        Self::new(item, HashSet::new())
    }
}

impl From<EnhancedListItem> for ListItem {
    fn from(item: EnhancedListItem) -> Self {
        item.item
    }
}

impl EnhancedListItem {
    /// Returns the underlying list item
    pub fn item(&self) -> &ListItem {
        &self.item
    }

    /// Indicates if this component has todo information
    pub fn is_todo(&self) -> bool {
        EnhancedListItemAttribute::todo_attributes()
            .iter()
            .any(|attr| self.attributes.contains(attr))
    }

    /// Returns percent complete in form of 0.0 == 0% and 1.0 == 100%. This
    /// is a calculated percentage based on the sublist items (if there are
    /// any) or the item itself.
    ///
    /// This will search through all sub list items, check if they have
    /// todo properties, and calculate a sum. If none of the sublists or
    /// any series of nested sublists contains todo items that are NOT
    /// rejected and this item does also not have todo progress,
    /// None will be returned.
    pub fn compute_todo_progress(&self) -> Option<f32> {
        self.item
            .contents()
            .iter()
            .fold(None, |acc, c| match c.component {
                ListItemContent::InlineContent(_) => acc,
                ListItemContent::List(list) => {
                    let (mut sum, mut count) =
                        list.items.iter().fold((0.0, 0), |acc, item| {
                            // NOTE: This is a recursive call that is NOT
                            //       tail recursive, but I do not want to
                            //       spend the time needed to translate it
                            //       into an interative approach given we
                            //       need to calculate the leaf todos before
                            //       determining the progress of the current
                            //       todo list item
                            if let Some(p) =
                                item.component.compute_todo_progress()
                            {
                                (acc.0 + p, acc.1 + 1)
                            } else {
                                acc
                            }
                        });

                    if let Some((acc_sum, acc_count)) = acc {
                        sum += acc_sum;
                        count += acc_count;
                    }

                    if count > 0 {
                        Some((sum, count))
                    } else {
                        None
                    }
                }
            })
            .map(|(sum, count)| sum / count as f32)
            .or_else(|| self.to_todo_progress())
    }

    /// Returns progress based on current todo status, or yields None if
    /// not a todo or is a rejected todo.
    ///
    /// Incomplete              == 0%
    /// Partially Complete 1    == 25%
    /// Partially Complete 2    == 50%
    /// Partially Complete 3    == 75%
    /// Completed               == 100%
    #[inline]
    fn to_todo_progress(&self) -> Option<f32> {
        if self.is_todo() && !self.is_todo_rejected() {
            Some(if self.is_todo_partially_complete_1() {
                0.25
            } else if self.is_todo_partially_complete_2() {
                0.5
            } else if self.is_todo_partially_complete_3() {
                0.75
            } else if self.is_todo_complete() {
                1.0
            } else {
                0.0
            })
        } else {
            None
        }
    }

    /// Indicates whether or not this component is incomplete, meaning no progress
    pub fn is_todo_incomplete(&self) -> bool {
        self.attributes
            .contains(&EnhancedListItemAttribute::TodoIncomplete)
    }

    /// Indicates whether or not this component is partially complete (any range)
    pub fn is_todo_partially_complete(&self) -> bool {
        self.is_todo_partially_complete_1()
            || self.is_todo_partially_complete_2()
            || self.is_todo_partially_complete_3()
    }

    /// Indicates whether or not this component is partially complete (1-33%)
    pub fn is_todo_partially_complete_1(&self) -> bool {
        self.attributes
            .contains(&EnhancedListItemAttribute::TodoPartiallyComplete1)
    }

    /// Indicates whether or not this component is partially complete (34-66%)
    pub fn is_todo_partially_complete_2(&self) -> bool {
        self.attributes
            .contains(&EnhancedListItemAttribute::TodoPartiallyComplete2)
    }

    /// Indicates whether or not this component is partially complete (67-99%)
    pub fn is_todo_partially_complete_3(&self) -> bool {
        self.attributes
            .contains(&EnhancedListItemAttribute::TodoPartiallyComplete3)
    }

    /// Indicates whether or not this component is complete
    pub fn is_todo_complete(&self) -> bool {
        self.attributes
            .contains(&EnhancedListItemAttribute::TodoComplete)
    }

    /// Indicates whether or not this component is rejected
    pub fn is_todo_rejected(&self) -> bool {
        self.attributes
            .contains(&EnhancedListItemAttribute::TodoRejected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! enhanced_item {
        () => {
            EnhancedListItem::new(
                Default::default(),
                vec![].iter().cloned().collect(),
            )
        };
        ($attr:ident) => {
            EnhancedListItem::new(
                Default::default(),
                vec![EnhancedListItemAttribute::$attr]
                    .iter()
                    .cloned()
                    .collect(),
            )
        };
        ($attr:ident, $($child:expr),+) => {
            EnhancedListItem::new(
                ListItem::Unordered(super::super::UnorderedListItem::new(
                    super::super::UnorderedListItemType::Hyphen,
                    0,
                    vec![super::super::ListItemContent::List(
                        super::super::super::List::new(vec![$($child),+]),
                    )],
                )),
                vec![EnhancedListItemAttribute::$attr]
                    .iter()
                    .cloned()
                    .collect(),
            )
        };
    }

    #[test]
    fn is_todo_should_return_true_if_contains_any_todo_attribute() {
        assert!(enhanced_item!(TodoIncomplete).is_todo());
        assert!(enhanced_item!(TodoPartiallyComplete1).is_todo());
        assert!(enhanced_item!(TodoPartiallyComplete2).is_todo());
        assert!(enhanced_item!(TodoPartiallyComplete3).is_todo());
        assert!(enhanced_item!(TodoComplete).is_todo());
        assert!(enhanced_item!(TodoRejected).is_todo());
    }

    #[test]
    fn is_todo_should_return_false_if_does_not_contain_any_todo_attribute() {
        assert!(!EnhancedListItem::default().is_todo());
    }

    #[test]
    fn compute_todo_progress_should_use_own_progress_if_no_children() {
        assert_eq!(
            Some(0.0),
            enhanced_item!(TodoIncomplete).compute_todo_progress()
        );
        assert_eq!(
            Some(0.25),
            enhanced_item!(TodoPartiallyComplete1).compute_todo_progress()
        );
        assert_eq!(
            Some(0.5),
            enhanced_item!(TodoPartiallyComplete2).compute_todo_progress()
        );
        assert_eq!(
            Some(0.75),
            enhanced_item!(TodoPartiallyComplete3).compute_todo_progress()
        );
        assert_eq!(
            Some(1.0),
            enhanced_item!(TodoComplete).compute_todo_progress()
        );
        assert_eq!(None, enhanced_item!(TodoRejected).compute_todo_progress());
        assert_eq!(None, EnhancedListItem::default().compute_todo_progress());
    }

    #[test]
    fn compute_todo_progress_should_use_children_progress_if_has_children() {
        // - [ ] <CALCULATING>
        //     - [-] N/A
        //     - [X] 100%
        //     - [.] 25%
        //     - [o] 50%
        //     - [O] 75%
        //     - [ ] 0%
        assert_eq!(
            enhanced_item!(
                TodoIncomplete,
                enhanced_item!(TodoRejected),
                enhanced_item!(TodoComplete),
                enhanced_item!(TodoPartiallyComplete1),
                enhanced_item!(TodoPartiallyComplete2),
                enhanced_item!(TodoPartiallyComplete3),
                enhanced_item!(TodoIncomplete)
            )
            .compute_todo_progress(),
            Some((1.0 + 0.25 + 0.5 + 0.75 + 0.0) / 5.0)
        );
    }

    #[test]
    fn compute_todo_progress_should_support_deeper_children() {
        assert_eq!(
            enhanced_item!(
                TodoIncomplete,
                enhanced_item!(
                    TodoRejected,
                    enhanced_item!(TodoRejected),
                    enhanced_item!(TodoComplete),
                    enhanced_item!(TodoPartiallyComplete1),
                    enhanced_item!(TodoPartiallyComplete2),
                    enhanced_item!(TodoPartiallyComplete3),
                    enhanced_item!(TodoIncomplete)
                )
            )
            .compute_todo_progress(),
            Some((1.0 + 0.25 + 0.5 + 0.75 + 0.0) / 5.0)
        );
    }

    #[test]
    fn is_todo_incomplete_should_return_true_if_has_incomplete_attribute() {
        assert!(enhanced_item!(TodoIncomplete).is_todo_incomplete());
    }

    #[test]
    fn is_todo_incomplete_should_return_false_if_missing_incomplete_attribute()
    {
        assert!(!enhanced_item!(TodoPartiallyComplete1).is_todo_incomplete());
        assert!(!enhanced_item!(TodoPartiallyComplete2).is_todo_incomplete());
        assert!(!enhanced_item!(TodoPartiallyComplete3).is_todo_incomplete());
        assert!(!enhanced_item!(TodoComplete).is_todo_incomplete());
        assert!(!enhanced_item!(TodoRejected).is_todo_incomplete());
        assert!(!EnhancedListItem::default().is_todo_incomplete());
    }

    #[test]
    fn is_todo_partially_complete_should_return_true_if_has_any_partially_complete_attribute(
    ) {
        assert!(
            enhanced_item!(TodoPartiallyComplete1).is_todo_partially_complete()
        );
        assert!(
            enhanced_item!(TodoPartiallyComplete2).is_todo_partially_complete()
        );
        assert!(
            enhanced_item!(TodoPartiallyComplete3).is_todo_partially_complete()
        );
    }

    #[test]
    fn is_todo_partially_complete_should_return_false_if_missing_all_partially_complete_attributes(
    ) {
        assert!(!enhanced_item!(TodoIncomplete).is_todo_partially_complete());
        assert!(!enhanced_item!(TodoComplete).is_todo_partially_complete());
        assert!(!enhanced_item!(TodoRejected).is_todo_partially_complete());
        assert!(!EnhancedListItem::default().is_todo_partially_complete());
    }

    #[test]
    fn is_todo_partially_complete_1_should_return_true_if_has_partially_complete_1_attribute(
    ) {
        assert!(enhanced_item!(TodoPartiallyComplete1)
            .is_todo_partially_complete_1());
    }

    #[test]
    fn is_todo_partially_complete_1_should_return_false_if_missing_partially_complete_1_attribute(
    ) {
        assert!(!enhanced_item!(TodoIncomplete).is_todo_partially_complete_1());
        assert!(!enhanced_item!(TodoPartiallyComplete2)
            .is_todo_partially_complete_1());
        assert!(!enhanced_item!(TodoPartiallyComplete3)
            .is_todo_partially_complete_1());
        assert!(!enhanced_item!(TodoComplete).is_todo_partially_complete_1());
        assert!(!enhanced_item!(TodoRejected).is_todo_partially_complete_1());
        assert!(!EnhancedListItem::default().is_todo_partially_complete_1());
    }

    #[test]
    fn is_todo_partially_complete_2_should_return_true_if_has_partially_complete_2_attribute(
    ) {
        assert!(enhanced_item!(TodoPartiallyComplete2)
            .is_todo_partially_complete_2());
    }

    #[test]
    fn is_todo_partially_complete_2_should_return_false_if_missing_partially_complete_2_attribute(
    ) {
        assert!(!enhanced_item!(TodoIncomplete).is_todo_partially_complete_2());
        assert!(!enhanced_item!(TodoPartiallyComplete1)
            .is_todo_partially_complete_2());
        assert!(!enhanced_item!(TodoPartiallyComplete3)
            .is_todo_partially_complete_2());
        assert!(!enhanced_item!(TodoComplete).is_todo_partially_complete_2());
        assert!(!enhanced_item!(TodoRejected).is_todo_partially_complete_2());
        assert!(!EnhancedListItem::default().is_todo_partially_complete_2());
    }

    #[test]
    fn is_todo_partially_complete_3_should_return_true_if_has_partially_complete_3_attribute(
    ) {
        assert!(enhanced_item!(TodoPartiallyComplete3)
            .is_todo_partially_complete_3());
    }

    #[test]
    fn is_todo_partially_complete_3_should_return_false_if_missing_partially_complete_3_attribute(
    ) {
        assert!(!enhanced_item!(TodoIncomplete).is_todo_partially_complete_3());
        assert!(!enhanced_item!(TodoPartiallyComplete1)
            .is_todo_partially_complete_3());
        assert!(!enhanced_item!(TodoPartiallyComplete2)
            .is_todo_partially_complete_3());
        assert!(!enhanced_item!(TodoComplete).is_todo_partially_complete_3());
        assert!(!enhanced_item!(TodoRejected).is_todo_partially_complete_3());
        assert!(!EnhancedListItem::default().is_todo_partially_complete_3());
    }

    #[test]
    fn is_todo_complete_should_return_true_if_has_complete_attribute() {
        assert!(enhanced_item!(TodoComplete).is_todo_complete());
    }

    #[test]
    fn is_todo_complete_should_return_false_if_missing_complete_attribute() {
        assert!(!enhanced_item!(TodoIncomplete).is_todo_complete());
        assert!(!enhanced_item!(TodoPartiallyComplete1).is_todo_complete());
        assert!(!enhanced_item!(TodoPartiallyComplete2).is_todo_complete());
        assert!(!enhanced_item!(TodoPartiallyComplete3).is_todo_complete());
        assert!(!enhanced_item!(TodoRejected).is_todo_complete());
        assert!(!EnhancedListItem::default().is_todo_complete());
    }

    #[test]
    fn is_todo_rejected_should_return_true_if_has_rejected_attribute() {
        assert!(enhanced_item!(TodoRejected).is_todo_rejected());
    }

    #[test]
    fn is_todo_rejected_should_return_false_if_missing_rejected_attribute() {
        assert!(!enhanced_item!(TodoIncomplete).is_todo_rejected());
        assert!(!enhanced_item!(TodoPartiallyComplete1).is_todo_rejected());
        assert!(!enhanced_item!(TodoPartiallyComplete2).is_todo_rejected());
        assert!(!enhanced_item!(TodoPartiallyComplete3).is_todo_rejected());
        assert!(!enhanced_item!(TodoComplete).is_todo_rejected());
        assert!(!EnhancedListItem::default().is_todo_rejected());
    }
}
