type ThemeName = 'dark' | 'light';
const STORAGE_KEY = 'aether:theme';

class ThemeStore {
  current = $state<ThemeName>('dark');

  init() {
    const saved = localStorage.getItem(STORAGE_KEY) as ThemeName | null;
    this.set(saved ?? 'dark');
  }

  set(name: ThemeName) {
    this.current = name;
    document.documentElement.setAttribute('data-theme', name);
    localStorage.setItem(STORAGE_KEY, name);
  }

  toggle() {
    this.set(this.current === 'dark' ? 'light' : 'dark');
  }
}

export const theme = new ThemeStore();