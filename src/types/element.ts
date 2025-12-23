export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface UIElement {
  id: string;
  name: string;
  controlType: string;
  automationId?: string;
  className?: string;
  xpath?: string;
  cssSelector?: string;
  bounds: Rect;
  screenshot?: string;
  processName?: string;
  windowTitle?: string;
  parentId?: string;
  children?: string[];
  attributes: Record<string, string>;
  createdAt: string;
}

export interface ElementLibrary {
  id: string;
  name: string;
  description?: string;
  elements: UIElement[];
  createdAt: string;
  updatedAt: string;
}

export interface ElementLocator {
  strategy: 'automationId' | 'name' | 'className' | 'xpath' | 'css' | 'image';
  value: string;
  fallback?: ElementLocator;
}
