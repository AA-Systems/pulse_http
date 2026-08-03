pub fn find_subslice(stack: &[u8], find: &[u8]) -> Option<usize> {
    stack.windows(find.len()).position(|window| window == find)
}
