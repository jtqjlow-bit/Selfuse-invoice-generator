/** ISO 4217 currencies our two users actually invoice in. Trim/add as needed. */
const CURRENCIES = [
  "MYR",
  "USD",
  "SGD",
  "CNY",
  "EUR",
  "GBP",
  "AUD",
  "JPY",
  "HKD",
];

interface Props {
  value: string;
  onChange: (v: string) => void;
  disabled?: boolean;
  className?: string;
}

export function CurrencySelect({ value, onChange, disabled, className }: Props) {
  return (
    <select
      className={className}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      disabled={disabled}
    >
      {!CURRENCIES.includes(value) && <option value={value}>{value}</option>}
      {CURRENCIES.map((c) => (
        <option key={c} value={c}>
          {c}
        </option>
      ))}
    </select>
  );
}
