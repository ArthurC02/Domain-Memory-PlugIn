export const MAXIMUM_LOANS_PER_MEMBER = 3;
export const LOAN_DAYS = 21;

export function mayBorrow(activeLoans: number): boolean {
  return activeLoans < MAXIMUM_LOANS_PER_MEMBER;
}
