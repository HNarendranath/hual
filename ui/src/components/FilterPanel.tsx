import { PhotoFilters, RangeFilter } from '../lib/ipc';
import { useState } from 'react';
import { formatExposureInput, parseExposureInput } from '../lib/utils';

type Field = 'iso' | 'fStop' | 'exposureTime' | 'focalLength';
type Bound = 'min' | 'max';


interface Props {
    filters: PhotoFilters;
    onChange: (newFilter: PhotoFilters) => void;
}

export function FilterPanel({ filters, onChange }: Props) {
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

    const handleExposureChange = (bound: Bound, value: number | null) => {
        onChange({
            ...filters,
            exposureTime: {
                ...filters.exposureTime,
                [bound]: value
            }
        });
    };

    return (
        <div className="filter-panel">
            <FilterRange label = "Focal Length" field="focalLength" range={filters.focalLength} onChange={handleChange} step={1} />
            <FilterRange label = "ISO" field="iso" range={filters.iso} onChange={handleChange} />
            <FilterRange label = "F-Stop" field="fStop" range={filters.fStop} onChange={handleChange} step={0.1} />
            <ExposureRangeInput label = "Exposure Time" range={filters.exposureTime} onChange={handleExposureChange} />
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
            <div className="filter-range-inputs">
                <input type="number" placeholder="min" step={step}
                    value={range.min ?? ''} onChange={(e) => onChange(field, 'min', e.target.value)} />
                <span className="filter-range-separator">-</span>
                <input type="number" placeholder="max" step={step}
                    value={range.max ?? ''} onChange={(e) => onChange(field, 'max', e.target.value)} />
            </div>
        </div>
    );
}


function ExposureRangeInput({ label, range, onChange }: {
    label: string; range: RangeFilter;
    onChange: (bound: Bound, value: number | null) => void;
}) {
    const [minText, setMinText] = useState(formatExposureInput(range.min));
    const [prevMin, setPrevMin] = useState(range.min);
    if (range.min !== prevMin) {
        setPrevMin(range.min);
        if (parseExposureInput(minText) !== range.min) {
            setMinText(formatExposureInput(range.min));
        }
    }

    const [maxText, setMaxText] = useState(formatExposureInput(range.max));
    const [prevMax, setPrevMax] = useState(range.max);
    if (range.max !== prevMax) {
        setPrevMax(range.max);
        if (parseExposureInput(maxText) !== range.max) {
            setMaxText(formatExposureInput(range.max));
        }
    }

    return (
        <div className="filter-range">
            <span className="filter-range-label">{label}</span>
            <div className="filter-range-inputs">
                <input type="text" placeholder="min"
                    value={minText}
                    onChange={(e) => {
                        setMinText(e.target.value);
                        onChange('min', parseExposureInput(e.target.value));
                    }} />
                <span className="filter-range-separator">-</span>
                <input type="text" placeholder="max"
                    value={maxText}
                    onChange={(e) => {
                        setMaxText(e.target.value);
                        onChange('max', parseExposureInput(e.target.value));
                    }} />
            </div>
        </div>
    );
}
