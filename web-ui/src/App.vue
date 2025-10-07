<script setup lang="ts">
import { ref } from "vue";

// CC:Tweaked color definitions
interface Color {
  name: string;
  hex: string;
  blit: string;
}

const colors: Color[] = [
  { name: "white", hex: "#F0F0F0", blit: "0" },
  { name: "orange", hex: "#F2B233", blit: "1" },
  { name: "magenta", hex: "#E57FD8", blit: "2" },
  { name: "lightBlue", hex: "#99B2F2", blit: "3" },
  { name: "yellow", hex: "#DEDE6C", blit: "4" },
  { name: "lime", hex: "#7FCC19", blit: "5" },
  { name: "pink", hex: "#F2B2CC", blit: "6" },
  { name: "gray", hex: "#4C4C4C", blit: "7" },
  { name: "lightGray", hex: "#999999", blit: "8" },
  { name: "cyan", hex: "#4C99B2", blit: "9" },
  { name: "purple", hex: "#B266E5", blit: "a" },
  { name: "blue", hex: "#3366CC", blit: "b" },
  { name: "brown", hex: "#7F664C", blit: "c" },
  { name: "green", hex: "#57A64E", blit: "d" },
  { name: "red", hex: "#CC4C4C", blit: "e" },
  { name: "black", hex: "#111111", blit: "f" },
];

// Canvas settings
const canvasWidth = ref(20);
const canvasHeight = ref(10);
const selectedColor = ref<Color>(colors[15]!); // Default to black

// Canvas data
const canvas = ref<string[][]>([]);

// Initialize canvas
const initCanvas = () => {
  const newCanvas: string[][] = [];
  for (let y = 0; y < canvasHeight.value; y++) {
    const row: string[] = [];
    for (let x = 0; x < canvasWidth.value; x++) {
      // Preserve existing data if available
      const existingRow = canvas.value[y];
      if (existingRow && existingRow[x] !== undefined) {
        row.push(existingRow[x]!);
      } else {
        row.push("0"); // Default to white
      }
    }
    newCanvas.push(row);
  }
  canvas.value = newCanvas;
};

// Initialize on load
initCanvas();

// Drawing state
const isDrawing = ref(false);

const startDrawing = () => {
  isDrawing.value = true;
};

const stopDrawing = () => {
  isDrawing.value = false;
};

const paintPixel = (x: number, y: number) => {
  const row = canvas.value[y];
  if (row && y >= 0 && y < canvas.value.length && x >= 0 && x < row.length) {
    canvas.value[y]![x] = selectedColor.value.blit;
  }
};

const onPixelClick = (x: number, y: number) => {
  paintPixel(x, y);
};

const onPixelEnter = (x: number, y: number) => {
  if (isDrawing.value) {
    paintPixel(x, y);
  }
};

// Clear canvas
const clearCanvas = () => {
  for (let y = 0; y < canvas.value.length; y++) {
    const row = canvas.value[y];
    if (row) {
      for (let x = 0; x < row.length; x++) {
        canvas.value[y]![x] = "0"; // Reset to white
      }
    }
  }
};

// Update canvas size
const updateCanvasSize = () => {
  initCanvas();
};

// Export to blit format
const exportedData = ref("");
const showExport = ref(false);

const exportToBlit = () => {
  const blitLines = canvas.value.map((row) => row.join(""));
  const blitString = blitLines.join("");
  exportedData.value = JSON.stringify(blitString, null, 2);
  showExport.value = true;
};

const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(exportedData.value);
    alert("Copied to clipboard!");
  } catch (err) {
    alert("Failed to copy to clipboard");
  }
};

// Send to server
const sendToServer = async () => {
  try {
    const blitLines = canvas.value.map((row) => row.join(""));
    const blitString = blitLines.join("");

    const response = await fetch(
      `http://localhost:8080/save-frame/${encodeURIComponent(blitString)}`,
      {
        method: "GET",
      }
    );

    if (response.ok) {
      alert("Successfully sent to server!");
    } else {
      alert("Failed to send to server");
    }
  } catch (err) {
    alert("Error sending to server: " + err);
  }
};

// Get color hex from blit character
const getColorHex = (blit: string): string => {
  const color = colors.find((c) => c.blit === blit);
  return color ? color.hex : "#FFFFFF";
};
</script>

<template>
  <div class="min-h-screen bg-gray-900 text-white p-8">
    <div class="max-w-7xl mx-auto">
      <h1 class="text-4xl font-bold mb-8 text-center">
        cc-monitor-control Web UI
      </h1>

      <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Left Panel: Canvas Controls -->
        <div class="lg:col-span-1 space-y-6">
          <!-- Canvas Size -->
          <div class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4">Canvas Size</h2>
            <div class="space-y-4">
              <div>
                <label class="block text-sm mb-2"
                  >Width: {{ canvasWidth }}</label
                >
                <input
                  v-model.number="canvasWidth"
                  type="range"
                  min="1"
                  max="82"
                  class="w-full"
                  @change="updateCanvasSize"
                />
              </div>
              <div>
                <label class="block text-sm mb-2"
                  >Height: {{ canvasHeight }}</label
                >
                <input
                  v-model.number="canvasHeight"
                  type="range"
                  min="1"
                  max="40"
                  class="w-full"
                  @change="updateCanvasSize"
                />
              </div>
            </div>
          </div>

          <!-- Color Palette -->
          <div class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4">Color Palette</h2>
            <div class="grid grid-cols-4 gap-2">
              <button
                v-for="color in colors"
                :key="color.blit"
                :style="{ backgroundColor: color.hex }"
                :class="[
                  'w-full aspect-square rounded-lg border-2 transition-all hover:scale-110',
                  selectedColor.blit === color.blit
                    ? 'border-blue-400 ring-2 ring-blue-400'
                    : 'border-gray-600',
                ]"
                :title="`${color.name} (${color.blit})`"
                @click="selectedColor = color"
              ></button>
            </div>
            <div class="mt-4 p-3 bg-gray-700 rounded">
              <p class="text-sm">
                Selected:
                <span class="font-semibold">{{ selectedColor.name }}</span>
                <span class="text-gray-400 ml-2"
                  >({{ selectedColor.blit }})</span
                >
              </p>
              <div
                class="w-full h-8 rounded mt-2"
                :style="{ backgroundColor: selectedColor.hex }"
              ></div>
            </div>
          </div>

          <!-- Actions -->
          <div class="bg-gray-800 rounded-lg p-6 space-y-3">
            <h2 class="text-xl font-semibold mb-4">Actions</h2>
            <button
              @click="sendToServer"
              class="w-full bg-blue-600 hover:bg-blue-700 px-4 py-3 rounded-lg font-semibold transition-colors"
            >
              Send to Server
            </button>
            <button
              @click="exportToBlit"
              class="w-full bg-green-600 hover:bg-green-700 px-4 py-3 rounded-lg font-semibold transition-colors"
            >
              Export to Blit Format
            </button>
            <button
              @click="clearCanvas"
              class="w-full bg-red-600 hover:bg-red-700 px-4 py-3 rounded-lg font-semibold transition-colors"
            >
              Clear Canvas
            </button>
          </div>
        </div>

        <!-- Right Panel: Canvas -->
        <div class="lg:col-span-2">
          <div class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-xl font-semibold mb-4">Canvas</h2>
            <div class="overflow-auto bg-gray-700 p-4 rounded-lg">
              <div
                class="inline-block border-2 border-gray-600 select-none"
                @mousedown="startDrawing"
                @mouseup="stopDrawing"
                @mouseleave="stopDrawing"
              >
                <div v-for="(row, y) in canvas" :key="y" class="flex">
                  <div
                    v-for="(pixel, x) in row"
                    :key="`${y}-${x}`"
                    :style="{ backgroundColor: getColorHex(pixel) }"
                    class="border border-gray-600 cursor-crosshair"
                    style="width: 20px; height: 20px"
                    @mousedown="onPixelClick(x, y)"
                    @mouseenter="onPixelEnter(x, y)"
                  ></div>
                </div>
              </div>
            </div>
            <p class="text-sm text-gray-400 mt-4">
              Click to paint a pixel, or click and drag to draw continuously
            </p>
          </div>

          <!-- Export Output -->
          <div v-if="showExport" class="bg-gray-800 rounded-lg p-6 mt-6">
            <div class="flex justify-between items-center mb-4">
              <h2 class="text-xl font-semibold">Exported Blit Format</h2>
              <button
                @click="showExport = false"
                class="text-gray-400 hover:text-white"
              >
                ✕
              </button>
            </div>
            <textarea
              v-model="exportedData"
              readonly
              class="w-full h-64 bg-gray-700 text-white p-4 rounded font-mono text-sm"
            ></textarea>
            <button
              @click="copyToClipboard"
              class="mt-3 w-full bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg font-semibold transition-colors"
            >
              Copy to Clipboard
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
input[type="range"] {
  accent-color: #3b82f6;
}
</style>
