pub fn view() -> String {
    "view".to_string()
}

#[cfg(test)]
mod tests {
    use super::view;

    #[test]
    fn returns_placeholder_view_output() {
        assert_eq!(view(), "view");
    }
}
