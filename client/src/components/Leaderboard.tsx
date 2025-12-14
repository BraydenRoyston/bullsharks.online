import { BullSharkActivity, AthleteStats } from '../types/activity';
import './Leaderboard.css';

interface LeaderboardProps {
  activities: BullSharkActivity[];
}

function filterPastWeekActivities(activities: BullSharkActivity[]): BullSharkActivity[] {
  const oneWeekAgo = new Date();
  oneWeekAgo.setDate(oneWeekAgo.getDate() - 7);

  return activities.filter(activity => {
    const activityDate = new Date(activity.date);
    return activityDate >= oneWeekAgo;
  });
}

function calculateAthleteStats(activities: BullSharkActivity[]): AthleteStats[] {
  const athleteMap = new Map<string, { totalMeters: number; count: number }>();

  activities.forEach(activity => {
    const athleteName = activity.athlete_name || 'Unknown Athlete';
    const distance = activity.distance || 0;

    if (!athleteMap.has(athleteName)) {
      athleteMap.set(athleteName, { totalMeters: 0, count: 0 });
    }

    const stats = athleteMap.get(athleteName)!;
    stats.totalMeters += distance;
    stats.count += 1;
  });

  const athleteStats: AthleteStats[] = Array.from(athleteMap.entries()).map(
    ([athleteName, { totalMeters, count }]) => ({
      athleteName,
      totalKilometers: totalMeters / 1000,
      activityCount: count,
    })
  );

  athleteStats.sort((a, b) => b.totalKilometers - a.totalKilometers);

  return athleteStats;
}

export function Leaderboard({ activities }: LeaderboardProps) {
  const pastWeekActivities = filterPastWeekActivities(activities);
  const athleteStats = calculateAthleteStats(pastWeekActivities);

  if (athleteStats.length === 0) {
    return (
      <div className="leaderboard">
        <h2 className="leaderboard-title">Weekly Leaderboard</h2>
        <p className="no-data">No running activities found in the past week.</p>
      </div>
    );
  }

  return (
    <div className="leaderboard">
      <h2 className="leaderboard-title">Weekly Leaderboard</h2>
      <p className="leaderboard-subtitle">Top runners from the past 7 days</p>

      <div className="leaderboard-table">
        <div className="leaderboard-header">
          <div className="rank-column">Rank</div>
          <div className="athlete-column">Athlete</div>
          <div className="distance-column">Distance (km)</div>
          <div className="activities-column">Activities</div>
        </div>

        {athleteStats.map((stats, index) => (
          <div key={stats.athleteName} className={`leaderboard-row ${index < 3 ? 'podium' : ''}`}>
            <div className="rank-column">
              {index === 0 && '🥇'}
              {index === 1 && '🥈'}
              {index === 2 && '🥉'}
              {index > 2 && `#${index + 1}`}
            </div>
            <div className="athlete-column">{stats.athleteName}</div>
            <div className="distance-column">{stats.totalKilometers.toFixed(2)}</div>
            <div className="activities-column">{stats.activityCount}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
