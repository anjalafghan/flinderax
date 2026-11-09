// Mutation helpers that take one argument, so pages and cards share the same invalidation.
import { usePeopleMutation, addTouch, type TouchInput } from "@/services/people";

export interface TouchArgs {
  personId: string;
  input: TouchInput;
}

export const useAddTouch = () => usePeopleMutation((a: TouchArgs) => addTouch(a.personId, a.input));
