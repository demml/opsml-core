use opsml_sql::schemas::schema::User;

pub fn has_permission(user: &User, permission: &str) -> bool {
    user.permissions.contains(&permission.to_string())
        || user.group_permissions.contains(&"admin".to_string())
}

pub fn has_read_permission(user: &User) -> bool {
    has_permission(user, "read")
}

pub fn has_write_permission(user: &User, repository_id: &str) -> bool {
    has_permission(user, &format!("write:{}", repository_id))
}

pub fn has_delete_permission(user: &User, repository_id: &str) -> bool {
    has_permission(user, &format!("delete:{}", repository_id))
}
