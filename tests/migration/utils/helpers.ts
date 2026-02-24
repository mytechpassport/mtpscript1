function validateEmail(email: string): boolean {
  return email.includes('@');
}

function formatUserName(first: string, last: string): string {
  return `${first} ${last}`;
}
