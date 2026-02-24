interface User {
    name: string;
    age: number;
    email: string | null;
}

class UserService {
    constructor() {}

    createUser(name: string, age: number): User {
        return {
            name: name,
            age: age,
            email: null
        };
    }
}
