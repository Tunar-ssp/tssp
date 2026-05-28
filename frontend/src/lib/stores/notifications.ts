import { writable } from 'svelte/store';

export interface Notification {
  id: string;
  type: 'success' | 'error' | 'info' | 'warning';
  title: string;
  message: string;
  duration?: number;
}

export const notifications = writable<Notification[]>([]);
const MAX_NOTIFICATIONS = 3;

export function addNotification(
  type: Notification['type'],
  title: string,
  message: string,
  duration: number = 3000
) {
  const id = Math.random().toString(36).substr(2, 9);
  const notification: Notification = { id, type, title, message, duration };

  notifications.update(n => {
    const next = [...n, notification];
    if (next.length > MAX_NOTIFICATIONS) {
      return next.slice(next.length - MAX_NOTIFICATIONS);
    }
    return next;
  });

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

export function success(titleOrMessage: string, message?: string) {
  const title = message ? titleOrMessage : 'Success';
  const msg = message || titleOrMessage;
  return addNotification('success', title, msg, 2000);
}

export function error(titleOrMessage: string, message?: string) {
  const title = message ? titleOrMessage : 'Error';
  const msg = message || titleOrMessage;
  return addNotification('error', title, msg, 5000);
}

export function info(titleOrMessage: string, message?: string) {
  const title = message ? titleOrMessage : 'Info';
  const msg = message || titleOrMessage;
  return addNotification('info', title, msg);
}

export function warning(titleOrMessage: string, message?: string) {
  const title = message ? titleOrMessage : 'Warning';
  const msg = message || titleOrMessage;
  return addNotification('warning', title, msg, 5000);
}
