import type { Account } from "@/services/types";
import { Select } from "@/components/ui/select";

/** Picks one of the user's accounts, or none. */
export function AccountSelect({
  accounts,
  value,
  onChange,
  id,
}: {
  accounts: Account[];
  value: string;
  onChange: (v: string) => void;
  id?: string;
}) {
  return (
    <Select id={id} value={value} onChange={(e) => onChange(e.target.value)}>
      <option value="">Any account</option>
      {accounts.map((a) => (
        <option key={a.account_id} value={a.account_id}>
          {a.name}
        </option>
      ))}
    </Select>
  );
}
