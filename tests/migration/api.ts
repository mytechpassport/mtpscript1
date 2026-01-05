interface APIResponse<T> {
    data: T;
    status: number;
    error: string | null;
}

function fetchUser(id: number): Promise<User> {
    return fetch(`/users/${id}`).then(r => r.json());
}

enum Status {
    Active = "active",
    Inactive = "inactive"
}
