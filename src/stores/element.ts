import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { UIElement, ElementLibrary } from '@/types';

export interface ElementLibraryInfo {
  id: string;
  name: string;
  description?: string;
  element_count: number;
  created_at: string;
  updated_at: string;
}

export const useElementStore = defineStore('element', () => {
  const libraries = ref<ElementLibrary[]>([]);
  const libraryList = ref<ElementLibraryInfo[]>([]);
  const currentLibrary = ref<ElementLibrary | null>(null);
  const selectedElement = ref<UIElement | null>(null);
  const isCapturing = ref(false);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Create a new library
  function createLibrary(name: string): ElementLibrary {
    const now = new Date().toISOString();
    const library: ElementLibrary = {
      id: crypto.randomUUID(),
      name,
      elements: [],
      createdAt: now,
      updatedAt: now,
    };
    libraries.value.push(library);
    currentLibrary.value = library;
    return library;
  }

  // Add element to current library
  function addElement(element: UIElement) {
    if (currentLibrary.value) {
      currentLibrary.value.elements.push(element);
      currentLibrary.value.updatedAt = new Date().toISOString();
    }
  }

  // Remove element from current library
  function removeElement(id: string) {
    if (currentLibrary.value) {
      currentLibrary.value.elements = currentLibrary.value.elements.filter(
        (e) => e.id !== id
      );
      currentLibrary.value.updatedAt = new Date().toISOString();
      if (selectedElement.value?.id === id) {
        selectedElement.value = null;
      }
    }
  }

  // Select an element
  function selectElement(element: UIElement | null) {
    selectedElement.value = element;
  }

  // Start capture mode
  function startCapture() {
    isCapturing.value = true;
  }

  // Stop capture mode
  function stopCapture() {
    isCapturing.value = false;
  }

  // Capture element at position
  async function captureElementAt(x: number, y: number): Promise<UIElement | null> {
    try {
      isCapturing.value = true;
      error.value = null;
      const element = await invoke<UIElement>('capture_element', { x, y });
      return element;
    } catch (e) {
      error.value = String(e);
      return null;
    } finally {
      isCapturing.value = false;
    }
  }

  // Save library to database
  async function saveLibrary(library?: ElementLibrary): Promise<string | null> {
    const lib = library || currentLibrary.value;
    if (!lib) return null;

    try {
      isLoading.value = true;
      error.value = null;
      const id = await invoke<string>('save_element_library', { library: lib });
      return id;
    } catch (e) {
      error.value = String(e);
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  // Load library from database
  async function loadLibrary(id: string): Promise<ElementLibrary | null> {
    try {
      isLoading.value = true;
      error.value = null;
      const library = await invoke<ElementLibrary>('load_element_library', { id });
      currentLibrary.value = library;

      // Update local list
      const existingIndex = libraries.value.findIndex(l => l.id === id);
      if (existingIndex >= 0) {
        libraries.value[existingIndex] = library;
      } else {
        libraries.value.push(library);
      }

      return library;
    } catch (e) {
      error.value = String(e);
      return null;
    } finally {
      isLoading.value = false;
    }
  }

  // List all libraries
  async function fetchLibraries(): Promise<ElementLibraryInfo[]> {
    try {
      isLoading.value = true;
      error.value = null;
      const list = await invoke<ElementLibraryInfo[]>('list_element_libraries');
      libraryList.value = list;
      return list;
    } catch (e) {
      error.value = String(e);
      return [];
    } finally {
      isLoading.value = false;
    }
  }

  // Delete library
  async function deleteLibrary(id: string): Promise<boolean> {
    try {
      isLoading.value = true;
      error.value = null;
      await invoke('delete_element_library', { id });

      // Update local lists
      libraries.value = libraries.value.filter(l => l.id !== id);
      libraryList.value = libraryList.value.filter(l => l.id !== id);

      if (currentLibrary.value?.id === id) {
        currentLibrary.value = null;
      }

      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      isLoading.value = false;
    }
  }

  // Highlight element on screen
  async function highlightElement(element: UIElement): Promise<void> {
    try {
      await invoke('highlight_element', { element });
    } catch (e) {
      console.error('Failed to highlight element:', e);
    }
  }

  return {
    libraries,
    libraryList,
    currentLibrary,
    selectedElement,
    isCapturing,
    isLoading,
    error,
    createLibrary,
    addElement,
    removeElement,
    selectElement,
    startCapture,
    stopCapture,
    captureElementAt,
    saveLibrary,
    loadLibrary,
    fetchLibraries,
    deleteLibrary,
    highlightElement,
  };
});
