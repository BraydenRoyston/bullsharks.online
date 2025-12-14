export interface BullSharkActivity {
  id: string;
  date: string;
  athlete_name: string | null;
  resource_state: number | null;
  name: string | null;
  distance: number | null;
  moving_time: number | null;
  elapsed_time: number | null;
  total_elevation_gain: number | null;
  sport_type: string | null;
  workout_type: number | null;
  device_name: string | null;
}

export interface AthleteStats {
  athleteName: string;
  totalKilometers: number;
  activityCount: number;
}
