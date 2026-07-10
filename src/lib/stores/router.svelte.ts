// Router minimo en memoria basado en runes. Sin dependencias, sin URL hashes:
// el estado de navegacion es reactivo y suficiente para una app de escritorio.
export type RouteId = 'home' | 'instances' | 'settings';

class Router {
  current = $state<RouteId>('home');

  navigate(id: RouteId) {
    this.current = id;
  }
}

export const router = new Router();