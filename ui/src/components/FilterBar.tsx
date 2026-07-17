import { PhotoFilters, RangeFilter } from '../lib/ipc';

type Field = 'iso' | 'fStop' | 'exposureTime';
type Bound = 'min' | 'max';


interface Props {
    filters: PhotoFilters;
    onChange: (newFilter: PhotoFilters) => void;
}

export function FilterBar({ filters, onChange }: Props) {
    const handleChange = (field: Field, bound: Bound, val: string) => {
        const parsedVal = val === '' ? null : Number(val);
        onChange({
            ...filters,
            [field]: {
                ...filters[field],
                [bound]: parsedVal
            }
        });

    };

    return (
        <div className="filter-bar">
            <FilterRange label = "ISO" field="iso" range={filters.iso} onChange={handleChange} />
            <FilterRange label = "F-Stop" field="fStop" range={filters.fStop} onChange={handleChange} step={0.1} />
            <FilterRange label = "Exposure Time" field="exposureTime" range={filters.exposureTime} onChange={handleChange} step={0.001} />
        </div>
    );
}

function FilterRange({ label, field, range, onChange, step }: { 
    label: string; field: Field; range: RangeFilter; 
    onChange: (field: Field, bound: Bound, val: string) => void; step?: number
}) {
    return (
        <div className="filter-range">
            <span className="filter-range-label">{label}</span>
            <input type="number" placeholder="min" step={step}
                value={range.min ?? ''} onChange={(e) => onChange(field, 'min', e.target.value)} />
            <span className="filter-range-separator">-</span>
            <input type="number" placeholder="max" step={step} 
                value={range.max ?? ''} onChange={(e) => onChange(field, 'max', e.target.value)} />
        </div>
    );
}
