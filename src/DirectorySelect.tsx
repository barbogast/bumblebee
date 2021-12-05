import { open } from '@tauri-apps/api/dialog';

type Props = {
  buttonLabel: string;
  onChange: (newValue: string) => void;
  value: string;
};

const DirectorySelect = ({ buttonLabel, onChange, value }: Props) => {
  return (
    <div style={{ display: 'flex', flexDirection: 'row', marginBottom: 10 }}>
      <button
        onClick={() =>
          open({ directory: true })
            .then((path) => onChange(path as string))
            .catch(console.error)
        }
        style={{ marginRight: 10 }}
      >
        {buttonLabel}
      </button>
      <input value={value} onChange={(e) => onChange(e.target.value)} style={{ flex: 1 }} />
    </div>
  );
};

export default DirectorySelect;
