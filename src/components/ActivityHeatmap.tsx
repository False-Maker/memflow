import { useEffect, useState } from 'react';
import CalendarHeatmap from 'react-calendar-heatmap';
import 'react-calendar-heatmap/dist/styles.css';
import { invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'react-tooltip';
import { useApp } from '../contexts/AppContext';

interface HeatmapData {
  date: string;
  count: number;
}

export default function ActivityHeatmap({ onClose }: { onClose?: () => void }) {
  const { searchActivities, dispatch } = useApp();
  const [data, setData] = useState<HeatmapData[]>([]);
  const [year] = useState(new Date().getFullYear());

  useEffect(() => {
    loadHeatmapData();
  }, [year]);

  const loadHeatmapData = async () => {
    try {
      const stats = await invoke<HeatmapData[]>('get_activity_heatmap_stats', { year });
      setData(stats);
    } catch (error) {
      console.error('Failed to load heatmap data:', error);
    }
  };

  const handleDayClick = (value: any) => {
    if (!value) return;
    
    // Parse date string (YYYY-MM-DD) to start/end timestamps
    const date = new Date(value.date);
    const fromTs = Math.floor(date.setHours(0, 0, 0, 0) / 1000);
    const toTs = Math.floor(date.setHours(23, 59, 59, 999) / 1000);

    // Trigger search in Timeline
    searchActivities({
      fromTs,
      toTs,
    });
    
    // Switch to Timeline view
    dispatch({ type: 'SET_VIEW', payload: 'timeline' });

    // Close modal if prop provided
    onClose?.();
  };

  return (
    <div className="p-4 bg-surface/30 rounded-lg border border-glass-border">
      <div className="flex justify-between items-center mb-4">
        <h3 className="text-neon-blue font-semibold text-sm">Activity Heatmap</h3>
        <div className="text-xs text-gray-500">{year}</div>
      </div>
      
      <div className="w-full overflow-x-auto">
        <div className="min-w-[600px]">
          <CalendarHeatmap
            startDate={new Date(`${year}-01-01`)}
            endDate={new Date(`${year}-12-31`)}
            values={data}
            classForValue={(value) => {
              if (!value) {
                return 'color-empty';
              }
              // Scale: 0-4
              const count = value.count;
              if (count === 0) return 'color-empty';
              if (count < 10) return 'color-scale-1';
              if (count < 30) return 'color-scale-2';
              if (count < 60) return 'color-scale-3';
              return 'color-scale-4';
            }}
            tooltipDataAttrs={((value: any) => {
              if (!value || !value.date) {
                return { 
                  'data-tooltip-id': 'heatmap-tooltip',
                  'data-tooltip-content': 'No activity' 
                };
              }
              return {
                'data-tooltip-id': 'heatmap-tooltip',
                'data-tooltip-content': `${value.date}: ${value.count} activities`,
              };
            }) as any}
            onClick={handleDayClick}
          />
          <Tooltip id="heatmap-tooltip" />
        </div>
      </div>

      <style>{`
        .react-calendar-heatmap text {
          font-size: 10px;
          fill: #666;
        }
        .react-calendar-heatmap .color-empty {
          fill: #1f2937; /* gray-800 */
        }
        .react-calendar-heatmap .color-scale-1 {
          fill: #1e3a8a; /* blue-900 */
        }
        .react-calendar-heatmap .color-scale-2 {
          fill: #1d4ed8; /* blue-700 */
        }
        .react-calendar-heatmap .color-scale-3 {
          fill: #2563eb; /* blue-600 */
        }
        .react-calendar-heatmap .color-scale-4 {
          fill: #00f3ff; /* neon-blue */
        }
      `}</style>
    </div>
  );
}
