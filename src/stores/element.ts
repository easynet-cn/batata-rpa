import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { UIElement, ElementLibrary } from '@/types';

export const useElementStore = defineStore('element', () => {
  const libraries = ref<ElementLibrary[]>([]);
  const currentLibrary = ref<ElementLibrary | null>(null);
  const selectedElement = ref<UIElement | null>(null);
  const isCapturing = ref(false);

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

  function addElement(element: UIElement) {
    if (currentLibrary.value) {
      currentLibrary.value.elements.push(element);
      currentLibrary.value.updatedAt = new Date().toISOString();
    }
  }

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

  function selectElement(element: UIElement | null) {
    selectedElement.value = element;
  }

  function startCapture() {
    isCapturing.value = true;
  }

  function stopCapture() {
    isCapturing.value = false;
  }

  return {
    libraries,
    currentLibrary,
    selectedElement,
    isCapturing,
    createLibrary,
    addElement,
    removeElement,
    selectElement,
    startCapture,
    stopCapture,
  };
});
