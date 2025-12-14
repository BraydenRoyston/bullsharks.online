import { BullSharkActivity } from '../types/activity';

const API_BASE_URL = '/api';

export async function fetchActivities(): Promise<BullSharkActivity[]> {
  try {
    console.log("Entering fetch activities")
    const response = await fetch(`${API_BASE_URL}/read`);

    if (!response.ok) {
      console.log("Failed to fetch activities")
      throw new Error(`Failed to fetch activities: ${response.statusText}`);
    }

    const activities: BullSharkActivity[] = await response.json();
    console.log(activities);
    return activities;
  } catch (error) {
    console.error('Error fetching activities:', error);
    throw error;
  }
}
