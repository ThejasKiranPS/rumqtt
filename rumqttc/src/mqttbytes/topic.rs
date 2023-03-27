#[derive(Debug)]
pub enum InvalidTopicError {
    EmptyTopic,
    ContainsNull,
    ContainsWildCards,
    TooLong,
}

/// Checks if a topic or topic filter has wildcards
pub fn has_wildcards(s: &str) -> bool {
    s.contains('+') || s.contains('#')
}
/// Checks if a topic is valid
pub fn valid_topic(topic: &str) -> Result<(), InvalidTopicError> {
    use InvalidTopicError::*;
    if topic.contains('+') {
        return Err(ContainsWildCards);
    }

    if topic.contains('#') {
        return Err(ContainsWildCards);
    }

    if topic.is_empty() {
        return Err(EmptyTopic);
    }

    if topic.contains("\0") {
        return Err(ContainsNull);
    }

    if topic.len() > 65535 {
        return Err(TooLong);
    }

    Ok(())
}

/// Checks if the filter is valid
///
/// <https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718106>
pub fn valid_filter(filter: &str) -> bool {
    if filter.is_empty() {
        return false;
    }

    let hirerarchy = filter.split('/').collect::<Vec<&str>>();
    if let Some((last, remaining)) = hirerarchy.split_last() {
        for entry in remaining.iter() {
            // # is not allowed in filter except as a last entry
            // invalid: sport/tennis#/player
            // invalid: sport/tennis/#/ranking
            if entry.contains('#') {
                return false;
            }

            // + must occupy an entire level of the filter
            // invalid: sport+
            if entry.len() > 1 && entry.contains('+') {
                return false;
            }
        }

        // only single '#" or '+' is allowed in last entry
        // invalid: sport/tennis#
        // invalid: sport/++
        if last.len() != 1 && (last.contains('#') || last.contains('+')) {
            return false;
        }
    }

    true
}

/// Checks if topic matches a filter. topic and filter validation isn't done here.
///
/// **NOTE**: 'topic' is a misnomer in the arg. this can also be used to match 2 wild subscriptions
/// **NOTE**: make sure a topic is validated during a publish and filter is validated
/// during a subscribe
pub fn matches(topic: &str, filter: &str) -> bool {
    if !topic.is_empty() && topic[..1].contains('$') {
        return false;
    }

    let mut topics = topic.split('/');
    let mut filters = filter.split('/');

    for f in filters.by_ref() {
        // "#" being the last element is validated by the broker with 'valid_filter'
        if f == "#" {
            return true;
        }

        // filter still has remaining elements
        // filter = a/b/c/# should match topci = a/b/c
        // filter = a/b/c/d should not match topic = a/b/c
        let top = topics.next();
        match top {
            Some(t) if t == "#" => return false,
            Some(_) if f == "+" => continue,
            Some(t) if f != t => return false,
            Some(_) => continue,
            None => return false,
        }
    }

    // topic has remaining elements and filter's last element isn't "#"
    if topics.next().is_some() {
        return false;
    }

    true
}

#[cfg(test)]
mod test {
    #[test]
    fn wildcards_are_detected_correctly() {
        assert!(!super::has_wildcards("a/b/c"));
        assert!(super::has_wildcards("a/+/c"));
        assert!(super::has_wildcards("a/b/#"));
    }

    #[test]
    fn topics_are_validated_correctly() {
        use super::valid_topic;
        use super::InvalidTopicError::*;

        assert!(matches!(valid_topic("+wrong"), Err(ContainsWildCards)));
        assert!(matches!(valid_topic("wro#ng"), Err(ContainsWildCards)));
        assert!(matches!(valid_topic("w/r/o/n/g+"), Err(ContainsWildCards)));
        assert!(matches!(
            valid_topic("wrong/#/path"),
            Err(ContainsWildCards)
        ));

        // All Topic Names and Topic Filters MUST be at least one character long
        assert!(matches!(valid_topic(""), Err(EmptyTopic)));
        assert!(matches!(valid_topic("_"), Ok(())));

        // Topic Names and Topic Filters MUST NOT include the null character (Unicode U+0000)
        assert!(matches!(
            valid_topic("string_with_null\0"),
            Err(ContainsNull)
        ));
        assert!(matches!(valid_topic("string_with_no_null\\0"), Ok(())));

        // Topic Names and Topic Filters are UTF-8 encoded strings,
        // they MUST NOT encode to more than 65535 bytes
        let invalid_topic = String::from_utf8(vec![97; 65535 + 1]).unwrap();
        assert!(matches!(super::valid_topic(&invalid_topic), Err(TooLong)));
        let valid_topic = String::from_utf8(vec![97; 65535]).unwrap();
        assert!(matches!(super::valid_topic(&valid_topic), Ok(())));
    }

    #[test]
    fn filters_are_validated_correctly() {
        assert!(!super::valid_filter("wrong/#/filter"));
        assert!(!super::valid_filter("wrong/wr#ng/filter"));
        assert!(!super::valid_filter("wrong/filter#"));
        assert!(super::valid_filter("correct/filter/#"));
        assert!(!super::valid_filter("wr/o+/ng"));
        assert!(!super::valid_filter("wr/+o+/ng"));
        assert!(!super::valid_filter("wron/+g"));
        assert!(super::valid_filter("cor/+/rect/+"));
    }

    #[test]
    fn zero_len_subscriptions_are_not_allowed() {
        assert!(!super::valid_filter(""));
    }

    #[test]
    fn dollar_subscriptions_doesnt_match_dollar_topic() {
        assert!(super::matches("sy$tem/metrics", "sy$tem/+"));
        assert!(!super::matches("$system/metrics", "$system/+"));
        assert!(!super::matches("$system/metrics", "+/+"));
    }

    #[test]
    fn topics_match_with_filters_as_expected() {
        let topic = "a/b/c";
        let filter = "a/b/c";
        assert!(super::matches(topic, filter));

        let topic = "a/b/c";
        let filter = "d/b/c";
        assert!(!super::matches(topic, filter));

        let topic = "a/b/c";
        let filter = "a/b/e";
        assert!(!super::matches(topic, filter));

        let topic = "a/b/c";
        let filter = "a/b/c/d";
        assert!(!super::matches(topic, filter));

        let topic = "a/b/c";
        let filter = "#";
        assert!(super::matches(topic, filter));

        let topic = "a/b/c";
        let filter = "a/b/c/#";
        assert!(super::matches(topic, filter));

        let topic = "a/b/c/d";
        let filter = "a/b/c";
        assert!(!super::matches(topic, filter));

        let topic = "a/b/c/d";
        let filter = "a/b/c/#";
        assert!(super::matches(topic, filter));

        let topic = "a/b/c/d/e/f";
        let filter = "a/b/c/#";
        assert!(super::matches(topic, filter));

        let topic = "a/b/c";
        let filter = "a/+/c";
        assert!(super::matches(topic, filter));
        let topic = "a/b/c/d/e";
        let filter = "a/+/c/+/e";
        assert!(super::matches(topic, filter));

        let topic = "a/b";
        let filter = "a/b/+";
        assert!(!super::matches(topic, filter));

        let filter1 = "a/b/+";
        let filter2 = "a/b/#";
        assert!(super::matches(filter1, filter2));
        assert!(!super::matches(filter2, filter1));

        let filter1 = "a/b/+";
        let filter2 = "#";
        assert!(super::matches(filter1, filter2));

        let filter1 = "a/+/c/d";
        let filter2 = "a/+/+/d";
        assert!(super::matches(filter1, filter2));
        assert!(!super::matches(filter2, filter1));
    }
}
