/// Causes `None` to be included as "NULL" in the SQL code instead of a value for a bind parameter.
///
/// This cause separate prepared statements to be created for the `Some` and `None` variants, which
/// results in better performance if one of the variants allows a more efficient query plan.
pub struct BindIfSome<T>(pub Option<T>);
