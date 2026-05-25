import { writable } from 'svelte/store';

export interface Notification {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  title: string;
  message: string;
  duration?: number;
}

export const notifications = writable<Notification[]>([]);

export function addNotification(
  type: Notification['type'],
  title: string,
  message: string,
  duration: number = 4000
) {
  const id = Math.random().toString(36).substr(2, 9);
  const notification: Notification = { id, type, title, message, duration };

  notifications.update(n => [...n, notification]);

  if (duration > 0) {
    setTimeout(() => {
      removeNotification(id);
    }, duration);
  }

  return id;
}

export function removeNotification(id: string) {
  notifications.update(n => n.filter(notif => notif.id !== id));
}

export function success(title: string, message: string) {
  return addNotification('success', title, message);
}

export function error(title: string, message: string) {
  return addNotification('error', title, message, 5000);
}

export function info(title: string, message: string) {
  return addNotification('info', title, message);
}

export function warning(title: string, message: string) {
  return addNotification('warning', title, message, 5000);
}
