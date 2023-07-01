export interface ItemMetadata {
  name: string;
  price: string;  // U128 in NEAR is used for large integer values, which can be represented as a string in TypeScript
  imgUrl: string;
  owner: string;  // AccountId in NEAR is a string that represents an account's unique ID
}

export interface Log {
  id: string;
  timestamp: number;  // u64 in Rust is a large integer, which can be represented as a number in TypeScript
  action: string;
  actor: string;
  entity: string;
  extra: string;
}