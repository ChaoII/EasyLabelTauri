<template>
  <div class="canvas-container"
       :class="{ 'is-dragging': drag.active, 'has-image': store.imageLoaded, 'hole-mode': holeModeActive }"
       ref="containerRef"
       @mousedown="onMouseDown"
       @mousemove="onMouseMove"
       @mouseup="onMouseUp"
       @mouseenter="onMouseEnter"
       @mouseleave="onMouseLeave"
       @wheel.prevent="onWheel"
       @dblclick="onDblClick"
       @contextmenu.prevent>
    <!-- 加载遮罩：切图瞬间先屏蔽旧图 -->
    <div v-if="isLoading" class="loading-overlay">
      <div class="loading-spinner" />
      <span class="loading-text">正在加载…</span>
    </div>

    <!-- OCR 模式指示 -->
    <div v-if="store.activeTool === 'ocr'" class="ocr-indicator">
      {{ ocrModeText }}
    </div>

    <!-- 十字线 -->
    <div ref="crosshairXRef" class="crosshair crosshair-x" v-show="showCrosshair"></div>
    <div ref="crosshairYRef" class="crosshair crosshair-y" v-show="showCrosshair"></div>

    <img v-show="store.imageLoaded && !isLoading" ref="imgRef" class="canvas-image"
         :src="imageSrc"
         :style="imgStyle" @load="onImageLoad" @error="onImageError" />
    <svg v-show="store.imageLoaded && !isLoading" class="annotation-svg"
         :style="svgStyle"
         :viewBox="`0 0 ${cw} ${ch}`">

      <!-- 已完成标注 -->
      <g v-for="ann in store.annotations" :key="ann.id"
        @mousedown.left.prevent="onAnnotationMousedown($event, ann)">
        <!-- AxisAlignedBox -->
        <template v-if="ann.type === 'AxisAlignedBox'">
          <rect
            :x="ann.x1 * cw"
            :y="ann.y1 * ch"
            :width="(ann.x2 - ann.x1) * cw"
            :height="(ann.y2 - ann.y1) * ch"
            :stroke="getClassColor(ann.class_id)"
            :stroke-width="ann.id === store.selectedAnnotationId ? 1.25 : 1"
            stroke-linejoin="miter"
            :fill="ann.id === store.selectedAnnotationId ? getClassColor(ann.class_id) + '28' : 'rgba(0,0,0,0)'" />
          <template v-for="L in [aabLabelLayout(ann)]" :key="ann.id + '-lbl'">
            <g class="ann-label">
              <rect
                :x="L.x"
                :y="L.y"
                :width="L.w"
                :height="LABEL_TAG_H"
                :fill="getClassColor(ann.class_id)" />
              <text
                :x="L.tx"
                :y="L.ty"
                fill="#ffffff"
                font-size="10"
                font-weight="500"
                text-anchor="middle"
                dominant-baseline="central">{{ getClassName(ann.class_id) }}</text>
            </g>
          </template>
          <template v-if="ann.id === store.selectedAnnotationId">
            <rect
              v-for="e in aabEdgeHandles(ann)"
              :key="e.h"
              :x="e.x - H_HALF"
              :y="e.y - H_HALF"
              :width="H_SZ" :height="H_SZ"
              fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
              :data-handle="e.h" :class="'handle handle-edge handle-edge-' + e.h" />
            <rect
              v-for="c in aabCornerHandles(ann)"
              :key="c.h"
              :x="c.x - H_HALF"
              :y="c.y - H_HALF"
              :width="H_SZ" :height="H_SZ"
              fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
              :data-handle="c.h" :class="'handle handle-corner handle-corner-' + c.h" />
          </template>
        </template>
        <!-- RotatedBox -->
        <template v-if="ann.type === 'RotatedBox'">
          <rect
            :x="ann.cx * cw - ann.width * cw / 2"
            :y="ann.cy * ch - ann.height * ch / 2"
            :width="ann.width * cw"
            :height="ann.height * ch"
            :stroke="getClassColor(ann.class_id)"
            :stroke-width="ann.id === store.selectedAnnotationId ? 1.25 : 1"
            stroke-linejoin="miter"
            :fill="ann.id === store.selectedAnnotationId ? getClassColor(ann.class_id) + '28' : 'rgba(0,0,0,0)'"
            :transform="`rotate(${ann.angle * 180 / Math.PI} ${ann.cx * cw} ${ann.cy * ch})`" />
          <template v-for="L in [rbLabelLayout(ann)]" :key="ann.id + '-rlbl'">
            <g class="ann-label">
              <rect
                :x="L.x"
                :y="L.y"
                :width="L.w"
                :height="LABEL_TAG_H"
                :fill="getClassColor(ann.class_id)" />
              <text
                :x="L.tx"
                :y="L.ty"
                fill="#ffffff"
                font-size="10"
                font-weight="500"
                text-anchor="middle"
                dominant-baseline="central">{{ getClassName(ann.class_id) }}</text>
            </g>
          </template>
          <template v-if="ann.id === store.selectedAnnotationId">
            <template v-for="hkey in ['tl','tr','bl','br']" :key="hkey">
              <rect
                :x="rbHandlePos(ann, hkey, cw, ch).x - H_HALF"
                :y="rbHandlePos(ann, hkey, cw, ch).y - H_HALF"
                :width="H_SZ" :height="H_SZ"
                fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
                :data-handle="hkey"
                :class="['handle', 'handle-rb-' + hkey]" />
            </template>
            <rect
              :x="ann.cx * cw - H_HALF"
              :y="ann.cy * ch - H_HALF"
              :width="H_SZ" :height="H_SZ"
              fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
              data-handle="move" class="move-handle" />
            <!-- 旋转手柄：顶部中心上方连线+圆圈 -->
            <line
              :x1="rbHandlePos(ann, 'tc', cw, ch).x"
              :y1="rbHandlePos(ann, 'tc', cw, ch).y"
              :x2="rbRotateHandlePos(ann, cw, ch).x"
              :y2="rbRotateHandlePos(ann, cw, ch).y"
              :stroke="getClassColor(ann.class_id)" stroke-width="1" />
            <circle
              :cx="rbRotateHandlePos(ann, cw, ch).x"
              :cy="rbRotateHandlePos(ann, cw, ch).y"
              r="3"
              fill="#ffffff" :stroke="getClassColor(ann.class_id)" stroke-width="1.5"
              data-handle="rotate"
              class="handle handle-rb-rotate" />
          </template>
        </template>
        <!-- Polygon -->
        <template v-if="ann.type === 'Polygon'">
          <path
            :d="polygonPath(ann)"
            :stroke="getClassColor(ann.class_id)"
            :stroke-width="ann.id === store.selectedAnnotationId ? 1.25 : 1"
            stroke-linejoin="miter"
            :fill="getClassColor(ann.class_id) + '28'"
            fill-rule="evenodd"
            style="pointer-events:none" />
          <!-- 外接矩形框 + 标签 -->
          <template v-for="B in [polyBBox(ann)]" :key="ann.id + '-pbbox'">
            <rect
              :x="B.x" :y="B.y" :width="B.w" :height="B.h"
              :stroke="getClassColor(ann.class_id)"
              :stroke-width="ann.id === store.selectedAnnotationId ? 1 : 0.8"
              fill="transparent"
              style="pointer-events:all" />
            <template v-for="L in [polyBBoxLabel(ann, B)]" :key="ann.id + '-pblbl'">
              <g class="ann-label">
                <rect :x="L.x" :y="L.y" :width="L.w" :height="LABEL_TAG_H" :fill="getClassColor(ann.class_id)" />
                <text :x="L.tx" :y="L.ty" fill="#ffffff" font-size="10" font-weight="500"
                  text-anchor="middle" dominant-baseline="central">{{ getClassName(ann.class_id) }}</text>
              </g>
            </template>
          </template>
          <template v-if="ann.id === store.selectedAnnotationId">
            <rect
              v-for="(p, i) in ann.points"
              :key="i"
              :x="p.x * cw - H_HALF"
              :y="p.y * ch - H_HALF"
              :width="H_SZ" :height="H_SZ"
              fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
              :data-handle="`poly-${i}`" class="handle handle-poly-vertex" />
            <template v-for="(hole, hi) in (ann.holes ?? [])" :key="'hole-' + hi">
              <rect
                v-for="(p, pi) in hole"
                :key="'h' + hi + '-' + pi"
                :x="p.x * cw - H_HALF"
                :y="p.y * ch - H_HALF"
                :width="H_SZ" :height="H_SZ"
                fill="#ffffff" stroke="#fbbf24" stroke-width="1.5"
                :data-handle="`h${hi}-${pi}`" class="handle handle-hpoly-vertex" />
            </template>
          </template>
          <!-- 孔洞校验警告 -->
          <template v-if="ann.holes && ann.holes.length > 0">
            <template v-for="V in [validatePolygonHoles(ann)]" :key="ann.id + '-holval'">
              <g v-if="!V.valid" class="ann-label">
                <rect :x="0" :y="0" :width="12" :height="12" fill="#f87171" rx="2" />
                <text x="6" y="7" fill="#fff" font-size="9" font-weight="700" text-anchor="middle" dominant-baseline="central">!</text>
              </g>
            </template>
          </template>
        </template>
        <!-- Keypoint -->
        <template v-if="ann.type === 'Keypoint'">
          <!-- 已保存的包围框（与预览风格一致，便于对照） -->
          <template v-if="(ann as KeypointAnnotation).bounding_box">
            <rect
              :x="(ann as KeypointAnnotation).bounding_box!.cx * cw - (ann as KeypointAnnotation).bounding_box!.width * cw / 2"
              :y="(ann as KeypointAnnotation).bounding_box!.cy * ch - (ann as KeypointAnnotation).bounding_box!.height * ch / 2"
              :width="(ann as KeypointAnnotation).bounding_box!.width * cw"
              :height="(ann as KeypointAnnotation).bounding_box!.height * ch"
              :transform="`rotate(${(ann as KeypointAnnotation).bounding_box!.angle * 180 / Math.PI} ${(ann as KeypointAnnotation).bounding_box!.cx * cw} ${(ann as KeypointAnnotation).bounding_box!.cy * ch})`"
              stroke="#fbbf24"
              :stroke-width="ann.id === store.selectedAnnotationId ? 2 : 1.5"
              fill="rgba(251,191,36,0.08)"
              :data-kp-ann-id="ann.id"
              class="kp-bbox" />
            <!-- 编辑手柄：当选中时显示 -->
            <template v-if="ann.id === store.selectedAnnotationId">
              <template v-for="hkey in ['tl','tc','tr','mr','br','bc','bl','ml']" :key="hkey">
                <rect
                  :x="kpRbHandlePos((ann as KeypointAnnotation).bounding_box!, hkey, cw, ch).x - H_HALF"
                  :y="kpRbHandlePos((ann as KeypointAnnotation).bounding_box!, hkey, cw, ch).y - H_HALF"
                  :width="H_SZ" :height="H_SZ"
                  fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
                  :data-handle="hkey"
                  :data-kp-ann-id="ann.id"
                  :class="['kp-kp-handle', `kp-kp-${hkey}`]" />
              </template>
              <rect
                :x="(ann as KeypointAnnotation).bounding_box!.cx * cw - H_HALF"
                :y="(ann as KeypointAnnotation).bounding_box!.cy * ch - H_HALF"
                :width="H_SZ" :height="H_SZ"
                fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
                data-handle="move"
                :data-kp-ann-id="ann.id"
                class="handle move-handle" />
            </template>
          </template>
          <!-- 关键点：每个点独立显示，按序号配色（区分顺序）���无连接线 -->
          <g v-for="(kp, i) in (ann as KeypointAnnotation).keypoints" :key="'kp-ann-' + ann.id + '-' + i">
            <!-- 关键点圆圈 -->
            <circle
              :cx="kp.x * cw"
              :cy="kp.y * ch"
              r="8"
              :fill="kpColorByIndex(i, store.classes.length, (ann as KeypointAnnotation).class_id)"
              :opacity="kp.visibility === 2 ? 1 : 0.5"
              stroke="#ffffff"
              stroke-width="2" />
            <!-- 序号标签 -->
            <circle
              :cx="kp.x * cw + 10"
              :cy="kp.y * ch - 10"
              r="8"
              fill="#ffffff"
              stroke="#1a1a1a"
              stroke-width="1" />
            <text
              :x="kp.x * cw + 10"
              :y="kp.y * ch - 10"
              fill="#1a1a1a"
              font-size="9"
              font-weight="700"
              text-anchor="middle"
              dominant-baseline="central">
              {{ i + 1 }}
            </text>
            <!-- 名称标签 -->
            <text
              :x="kp.x * cw + 20"
              :y="kp.y * ch - 8"
              :fill="kpColorByIndex(i, store.classes.length, (ann as KeypointAnnotation).class_id)"
              font-size="11"
              font-weight="600"
              dominant-baseline="central">{{ kp.name }}</text>
            <!-- 可见性符号 -->
            <template v-if="kp.visibility !== 2">
              <text
                :x="kp.x * cw"
                :y="kp.y * ch + 14"
                fill="#6b7280"
                font-size="9"
                text-anchor="middle">
                {{ kp.visibility === 0 ? 'x 隐藏' : '~ 遮挡' }}
              </text>
            </template>
            <!-- 关键点顶点手柄：当选中标注时显示 -->
            <template v-if="ann.id === store.selectedAnnotationId">
              <rect
                :x="kp.x * cw - H_HALF"
                :y="kp.y * ch - H_HALF"
                :width="H_SZ" :height="H_SZ"
                fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
                :data-handle="`kp-${i}`"
                :data-kp-ann-id="ann.id"
                class="handle handle-kp-vertex" />
            </template>
          </g>
          <!-- 类别标签（显示在所有关键点左上方） -->
          <template v-for="L in [keypointLabelLayout(ann as KeypointAnnotation)]" :key="ann.id + '-klbl'">
            <g class="ann-label">
              <rect
                :x="L?.x ?? 0"
                :y="L?.y ?? 0"
                :width="L?.w ?? 30"
                :height="LABEL_TAG_H"
                :fill="getClassColor(ann.class_id)" />
              <text
                :x="L?.tx ?? 0"
                :y="L?.ty ?? 0"
                fill="#ffffff"
                font-size="10"
                font-weight="500"
                text-anchor="middle"
                dominant-baseline="central">{{ getClassName(ann.class_id) }}</text>
            </g>
          </template>
        </template>
        <!-- Ocr -->
        <template v-if="ann.type === 'Ocr'">
          <polygon
            :points="(ann as OcrAnnotation).points.map((p: Point) => `${p.x * cw},${p.y * ch}`).join(' ')"
            :stroke="getClassColor(ann.class_id)"
            :stroke-width="ann.id === store.selectedAnnotationId ? 1.5 : 1"
            stroke-linejoin="miter"
            fill="rgba(255,255,0,0.1)" />
          <!-- OCR 文本标签（显示在框左上方） -->
          <template v-for="L in [ocrLabelLayout(ann as OcrAnnotation)]" :key="ann.id + '-ocrlbl'">
            <g class="ann-label">
              <rect :x="L.x" :y="L.y" :width="L.w" :height="LABEL_TAG_H" :fill="getClassColor(ann.class_id)" rx="2" />
              <text :x="L.x + 4" :y="L.ty" fill="#ffffff" font-size="10" font-weight="500"
                text-anchor="start" dominant-baseline="central">{{ (ann as OcrAnnotation).text || getClassName(ann.class_id) }}</text>
            </g>
          </template>
          <!-- 编辑手柄：选中时显示 -->
          <template v-if="ann.id === store.selectedAnnotationId">
            <template v-if="isOcrRectMode(ann as OcrAnnotation)">
              <!-- 矩形模式：box 风格角点+中心移动手柄 -->
              <rect v-for="hkey in ['tl','tr','bl','br']" :key="hkey"
                :x="ocrHandlePos(ann, hkey).x - H_HALF"
                :y="ocrHandlePos(ann, hkey).y - H_HALF"
                :width="H_SZ" :height="H_SZ"
                fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
                :data-handle="'ocr-resize-' + hkey"
                :class="['handle', 'handle-rb-' + hkey]" />
              <rect
                :x="ocrCenter(ann).x - H_HALF"
                :y="ocrCenter(ann).y - H_HALF"
                :width="H_SZ" :height="H_SZ"
                fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
                data-handle="move" class="handle move-handle" />
            </template>
            <template v-else>
              <!-- 四边形模式：独立顶点手柄 -->
              <rect
                v-for="(p, i) in (ann as OcrAnnotation).points"
                :key="'ocr-pt-' + i"
                :x="p.x * cw - H_HALF"
                :y="p.y * ch - H_HALF"
                :width="H_SZ" :height="H_SZ"
                fill="#ffffff" stroke="#1a1a1a" stroke-width="1"
                :data-handle="`ocr-vertex-${i}`"
                class="handle handle-poly-vertex" />
            </template>
          </template>
        </template>
      </g>

      <!-- rotated_box preview -->
      <template v-if="store.activeTool === 'rotated_box'">
        <!-- 步骤1：画第一个点 -->
        <template v-if="rbStep === 1 && rbPt1">
          <circle :cx="rbPt1.x" :cy="rbPt1.y" r="5" :fill="drawingColor" stroke="white" stroke-width="2" />
          <line v-if="draw.curX !== null && draw.curY !== null" :x1="rbPt1.x" :y1="rbPt1.y" :x2="draw.curX" :y2="draw.curY" :stroke="drawingColor" stroke-width="1.5" stroke-dasharray="3,3" opacity="0.8" />
        </template>
        <!-- 步骤2：拖拽确定角度 + 宽度 -->
        <template v-if="rbStep === 2">
          <circle :cx="rbPt1.x" :cy="rbPt1.y" r="5" :fill="drawingColor" stroke="white" stroke-width="2" />
          <circle v-if="rbPt2" :cx="rbPt2.x" :cy="rbPt2.y" r="5" :fill="drawingColor" stroke="white" stroke-width="2" />
          <line v-if="rbPt2" :x1="rbPt1.x" :y1="rbPt1.y" :x2="rbPt2.x" :y2="rbPt2.y" :stroke="drawingColor" stroke-width="2" opacity="0.9" />
          <line v-if="!rbPt2 && draw.curX !== null && draw.curY !== null" :x1="rbPt1.x" :y1="rbPt1.y" :x2="draw.curX" :y2="draw.curY" :stroke="drawingColor" stroke-width="1.5" stroke-dasharray="3,3" opacity="0.8" />
        </template>
        <!-- 步骤3：画高度 -->
        <template v-if="rbStep === 3 && rbPt1 && rbPt2">
          <circle :cx="rbPt1.x" :cy="rbPt1.y" r="5" :fill="drawingColor" stroke="white" stroke-width="2" />
          <circle :cx="rbPt2.x" :cy="rbPt2.y" r="5" :fill="drawingColor" stroke="white" stroke-width="2" />
          <line :x1="rbPt1.x" :y1="rbPt1.y" :x2="rbPt2.x" :y2="rbPt2.y" :stroke="drawingColor" stroke-width="2" opacity="0.9" />
          <line v-if="rbStep3PerpLine" :x1="rbStep3PerpLine.x1" :y1="rbStep3PerpLine.y1" :x2="rbStep3PerpLine.x2" :y2="rbStep3PerpLine.y2" :stroke="drawingColor" stroke-width="1.5" stroke-dasharray="3,3" opacity="0.8" />
          <!-- 实时预览旋转矩形 -->
          <rect
            v-if="rbFinalPreview"
            :x="rbFinalPreview.x" :y="rbFinalPreview.y"
            :width="rbFinalPreview.w" :height="rbFinalPreview.h"
            :transform="`rotate(${rbFinalPreview.angle} ${rbFinalPreview.cx} ${rbFinalPreview.cy})`"
            :stroke="drawingColor" stroke-width="1" stroke-dasharray="4,3" :fill="drawingColor + '0d'" />
        </template>
      </template>

      <!-- polygon preview -->
      <template v-if="store.activeTool === 'polygon' && store.drawingPoints.length > 0">
        <polyline :points="store.drawingPoints.map((p: Point) => `${p.x * cw},${p.y * ch}`).join(' ')" :stroke="drawingColor" stroke-width="1" fill="none" />
        <circle :cx="store.drawingPoints[0].x * cw" :cy="store.drawingPoints[0].y * ch"
          :r="polylineNearFirst ? 9 : 4"
          :fill="drawingColor"
          stroke="white" :stroke-width="polylineNearFirst ? 2 : 0" />
        <circle v-for="(p, i) in store.drawingPoints.slice(1)" :key="i" :cx="p.x * cw" :cy="p.y * ch" r="4" :fill="drawingColor" />
        <line v-if="polyFollowLine" :x1="polyFollowLine.x1" :y1="polyFollowLine.y1" :x2="polyFollowLine.x2" :y2="polyFollowLine.y2" :stroke="drawingColor" stroke-width="1.5" stroke-dasharray="3,3" opacity="0.6" />
      </template>

      <!-- keypoint preview — 角点+矩形框模式 -->
      <template v-if="store.activeTool === 'keypoint' && store.kpPhase !== null">
        <!-- 阶段1: 角点预览（无连接线，独立显示每个点） -->
        <template v-if="store.kpPhase === 'corners'">
          <!-- 角点圆圈 + 序号标签 + 可见性 -->
          <g v-for="(cp, i) in store.keypointDrawing" :key="'kp-c-' + i">
            <!-- 圆圈 -->
            <circle
              :cx="cp.x * cw"
              :cy="cp.y * ch"
              r="10"
              :fill="kpColorByIndex(i, store.classes.length, props.selectedClassId)"
              :opacity="cp.visibility === 2 ? 1 : 0.6"
              stroke="#ffffff"
              stroke-width="2" />
            <!-- 可见性覆盖符号 -->
            <template v-if="cp.visibility !== 2">
              <text
                :x="cp.x * cw"
                :y="cp.y * ch + 14"
                fill="#6b7280"
                font-size="9"
                text-anchor="middle">
                {{ cp.visibility === 0 ? 'x 隐藏' : '~ 遮挡' }}
              </text>
            </template>
            <!-- 序号 -->
            <circle :cx="cp.x * cw + 10" :cy="cp.y * ch - 10" r="8" fill="#ffffff" stroke="#1a1a1a" stroke-width="1" />
            <text :x="cp.x * cw + 10" :y="cp.y * ch - 10" fill="#1a1a1a" font-size="9" font-weight="700" text-anchor="middle" dominant-baseline="central">
              {{ i + 1 }}
            </text>
            <!-- 名称标签 -->
            <text
              :x="cp.x * cw + 20"
              :y="cp.y * ch - 10"
              :fill="kpColorByIndex(i, store.classes.length, props.selectedClassId)"
              font-size="11"
              font-weight="600"
              dominant-baseline="central"
              :style="{ textShadow: '0 1px 4px rgba(0,0,0,0.9)' }">
              {{ cp.name }}
            </text>
          </g>
          <!-- 下一个待放置角点提示 -->
          <template v-if="store.keypointDrawing.length < store.currentKeypointNames.length">
            <circle
              v-if="draw.curX !== null && draw.curY !== null"
              :cx="draw.curX"
              :cy="draw.curY"
              r="10"
              :fill="drawingColor + '33'"
              :stroke="pendingKpVisibility === 0 ? '#6b7280' : pendingKpVisibility === 1 ? '#fbbf24' : drawingColor"
              stroke-width="2"
              stroke-dasharray="4,3" />
            <text
              v-if="draw.curX !== null && draw.curY !== null"
              :x="draw.curX"
              :y="draw.curY - 16"
              :fill="drawingColor"
              font-size="11"
              font-weight="600"
              text-anchor="middle">{{ store.currentKeypointNames[store.keypointDrawing.length] }}</text>
            <text
              v-if="draw.curX !== null && draw.curY !== null"
              :x="draw.curX"
              :y="draw.curY + 22"
              fill="#9ca3af"
              font-size="9"
              text-anchor="middle">v={{ pendingKpVisibility }} (0隐藏/1遮挡/2可见)</text>
          </template>
        </template>

        <!-- 阶段2: 包围框预览（跟随鼠标拖拽绘制） -->
        <template v-if="store.kpPhase === 'box'">
          <!-- 显示已放置的角点 -->
          <g v-for="(cp, i) in store.keypointDrawing" :key="'kp-c-fixed-' + i">
            <circle
              :cx="cp.x * cw"
              :cy="cp.y * ch"
              r="10"
              :fill="kpColorByIndex(i, store.classes.length, props.selectedClassId)"
              :opacity="cp.visibility === 2 ? 1 : 0.6"
              stroke="#ffffff"
              stroke-width="2" />
            <circle :cx="cp.x * cw + 12" :cy="cp.y * ch - 12" r="8" fill="#ffffff" stroke="#1a1a1a" stroke-width="1" />
            <text :x="cp.x * cw + 12" :y="cp.y * ch - 12" fill="#1a1a1a" font-size="9" font-weight="700" text-anchor="middle" dominant-baseline="central">{{ i + 1 }}</text>
            <text :x="cp.x * cw + 22" :y="cp.y * ch - 12" :fill="kpColorByIndex(i, store.classes.length, props.selectedClassId)" font-size="11" font-weight="600" dominant-baseline="central">{{ cp.name }}</text>
          </g>
          <!-- 矩形框预览（跟随鼠标拖拽实时更新） -->
          <template v-if="store.kpBoxPreview">
            <rect
              :x="Math.min(store.kpBoxPreview.x1 * cw, store.kpBoxPreview.x2 * cw)"
              :y="Math.min(store.kpBoxPreview.y1 * ch, store.kpBoxPreview.y2 * ch)"
              :width="Math.abs(store.kpBoxPreview.x2 - store.kpBoxPreview.x1) * cw"
              :height="Math.abs(store.kpBoxPreview.y2 - store.kpBoxPreview.y1) * ch"
              stroke="#fbbf24"
              stroke-width="2"
              fill="rgba(251,191,36,0.1)" />
          </template>
          <!-- 操作提示 -->
          <text
            v-if="draw.curX !== null && draw.curY !== null"
            :x="draw.curX"
            :y="draw.curY - 20"
            fill="#fbbf24"
            font-size="12"
            font-weight="600"
            text-anchor="middle"
            :style="{ textShadow: '0 1px 4px rgba(0,0,0,0.9)' }">
            拖拽绘制包围框
          </text>
        </template>
      </template>

      <!-- ocr preview — 四边形 + 文本输入 -->
      <template v-if="store.activeTool === 'ocr' && store.ocrDrawingPoints.length > 0">
        <polygon
          v-if="ocrTextInputVisible"
          :points="store.ocrDrawingPoints.map((p: Point) => `${p.x * cw},${p.y * ch}`).join(' ')"
          :stroke="drawingColor"
          stroke-width="1.5"
          :fill="drawingColor + '0d'"
          stroke-dasharray="4,3"
          stroke-linejoin="round" />
        <polyline
          v-else
          :points="store.ocrDrawingPoints.map((p: Point) => `${p.x * cw},${p.y * ch}`).join(' ')"
          :stroke="drawingColor"
          stroke-width="1.5"
          fill="none"
          stroke-dasharray="4,3"
          stroke-linejoin="round" />
        <!-- 所有顶点圆点 -->
        <circle
          v-for="(p, i) in store.ocrDrawingPoints"
          :key="'ocr-preview-pt-' + i"
          :cx="p.x * cw"
          :cy="p.y * ch"
          r="4"
          :fill="drawingColor"
          stroke="#ffffff"
          stroke-width="1.5" />
        <!-- 四边形模式下第一个点放大（鼠标靠近时） -->
        <circle
          v-if="!store.ocrRectMode && store.ocrDrawingPoints.length >= 2"
          :cx="store.ocrDrawingPoints[0].x * cw"
          :cy="store.ocrDrawingPoints[0].y * ch"
          :r="ocrNearFirst ? 9 : 5"
          :fill="drawingColor"
          stroke="white" :stroke-width="ocrNearFirst ? 2 : 0" />
        <!-- 跟随鼠标的预览线 -->
        <line
          v-if="!store.ocrRectMode && !ocrTextInputVisible && ocrFollowLine"
          :x1="ocrFollowLine.x1"
          :y1="ocrFollowLine.y1"
          :x2="ocrFollowLine.x2"
          :y2="ocrFollowLine.y2"
          :stroke="drawingColor"
          stroke-width="1.5"
          stroke-dasharray="4,3"
          opacity="0.5" />
        <!-- 顶点序号 -->
        <text
          v-for="(p, i) in store.ocrDrawingPoints"
          :key="'ocr-pt-label-' + i"
          :x="p.x * cw + 10"
          :y="p.y * ch + 4"
          :fill="drawingColor"
          font-size="10"
          font-weight="600">{{ i + 1 }}</text>
      </template>

      <!-- 分类工具覆盖层 -->
      <template v-if="store.activeTool === 'classification'">
        <g v-if="selectedClassificationIds.length > 0">
          <rect
            v-for="(id, i) in selectedClassificationIds"
            :key="id"
            :x="8"
            :y="cw > 500 ? 8 + i * 30 : 8 + i * 26"
            :width="120"
            :height="cw > 500 ? 26 : 22"
            rx="5"
            :fill="getClassColor(id)"
            opacity="0.9"
          />
          <text
            v-for="(id, i) in selectedClassificationIds"
            :key="id"
            :x="68"
            :y="cw > 500 ? 8 + i * 30 + 13 : 8 + i * 26 + 11"
            fill="#ffffff"
            font-size="10"
            font-weight="500"
            text-anchor="middle">{{ getClassName(id) }}</text>
        </g>
        <g v-else>
          <rect :x="cw / 2 - 90" :y="ch / 2 - 18" width="180" height="36" rx="6" fill="rgba(0,0,0,0.55)" />
          <text :x="cw / 2" :y="ch / 2 + 5" fill="#cccccc" font-size="13" text-anchor="middle" font-weight="500">请在右侧勾选类别</text>
        </g>
      </template>

    </svg>
    <!-- OCR 文本输入弹窗 -->
    <div v-if="ocrTextInputVisible" class="ocr-text-overlay" @click.self="ocrTextInputVisible = false">
      <div class="ocr-text-modal">
        <div class="ocr-text-title">输入 OCR 文本</div>
        <NInput
          ref="ocrInputRef"
          v-model:value="ocrTextInput"
          placeholder="请输入识别到的文本内容..."
          size="small"
          :maxlength="200"
          @keydown.enter="confirmOcrText"
          @keydown.escape="ocrTextInputVisible = false"
        />
        <div class="ocr-text-actions">
          <NButton size="small" quaternary @click="ocrTextInputVisible = false">取消</NButton>
          <NButton size="small" type="primary" @click="confirmOcrText">确认</NButton>
        </div>
      </div>
    </div>
    <div v-if="!store.imageLoaded" class="empty-state">
      <div
        class="drop-zone"
        role="button"
        tabindex="0"
        aria-label="拖放或点击选择图片"
        @click="emit('open-image')"
        @keydown.enter.prevent="emit('open-image')"
        @keydown.space.prevent="emit('open-image')"
      >
        <div class="icon">📁</div>
        <div class="title">拖放图片到此处开始标注</div>
        <div class="sub">或点击此处选择文件</div>
        <div class="formats">支持 JPG / PNG / BMP / WebP / TIFF</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from "vue";

const min = Math.min, max = Math.max;

import { useAppStore } from "@/stores/app";
import { convertFileSrc } from "@tauri-apps/api/core";
import { NInput, NButton } from "naive-ui";
import type { Annotation, AxisAlignedBox, ClassificationAnnotation, Point, PolygonAnnotation, RotatedBox, KeypointAnnotation, OcrAnnotation } from "@/utils/types";

const props = defineProps<{
  selectedClassId: number;
  taskType?: string;
  classes?: any[];
  images?: any[];
}>();
const emit = defineEmits<{ (e: "open-image"): void }>();

// 关键点绘制：当前待放置角点的可见性（0=隐藏 1=遮挡 2=可见）
const pendingKpVisibility = ref<0 | 1 | 2>(2);

const store = useAppStore();
const containerRef = ref<HTMLDivElement | null>(null);
const imgRef = ref<HTMLImageElement | null>(null);
const crosshairXRef = ref<HTMLDivElement | null>(null);
const crosshairYRef = ref<HTMLDivElement | null>(null);
const showCrosshair = ref(false);

function updateCrosshair(clientX: number, clientY: number) {
  const el = containerRef.value;
  if (!el || !crosshairXRef.value || !crosshairYRef.value) return;
  const rect = el.getBoundingClientRect();
  const x = clientX - rect.left;
  const y = clientY - rect.top;
  crosshairXRef.value.style.left = "0";
  crosshairXRef.value.style.top = y + "px";
  crosshairXRef.value.style.width = "100%";
  crosshairYRef.value.style.left = x + "px";
  crosshairYRef.value.style.top = "0";
  crosshairYRef.value.style.height = "100%";
}

// 图片自然像素尺寸
const cw = ref(0);
const ch = ref(0);

// ==================== 加载遮罩（防止切图时闪旧图） ====================
const isLoading = ref(false);
const lastLoadedPath = ref("");

// 直接用 store.imagePath（已经是文件绝对路径），通过 convertFileSrc 转为 webview 可访问 URL
const imageSrc = computed(() =>
  store.imagePath ? convertFileSrc(store.imagePath) : ""
);

// 监听图片路径变化：切图时先显示遮罩，新图片加载完成后自动隐藏
watch(() => store.imagePath, async (newPath) => {
  if (!newPath) {
    isLoading.value = false;
    return;
  }
  if (newPath === lastLoadedPath.value) return;
  isLoading.value = true;
  // 新图渲染完成后 onImageLoad 会将 isLoading 设为 false
});

const holePolygonId = ref<string | null>(null);
const holeModeActive = ref(false);

watch(() => store.activeTool, () => {
  holePolygonId.value = null;
  holeModeActive.value = false;
});

// ==================== 监听：切换类别时重置关键点绘制状态 ====================
// 当选择的类别变化时（kpNames 随之变化），重置当前绘制中的内容
// 这样不同 keypoint_names 数量的类别之间切换不会导致角点错位
watch(
  () => props.selectedClassId,
  () => {
    if (store.activeTool !== "keypoint") return;
    // 重置绘制状态：回到 corners 阶段，等待用户开始新的标注
    store.resetKeypointDraw();
  }
);

// ==================== 绘图状态 ====================

const draw = reactive({
  startX: null as number | null,
  startY: null as number | null,
  curX: null as number | null,
  curY: null as number | null,
});

const boxStartX = ref(0);
const boxStartY = ref(0);
let boxPreviewDiv: HTMLDivElement | null = null;

function startBoxPreview(x: number, y: number) {
  const el = containerRef.value;
  if (!el) return;
  removeBoxPreview();
  const div = document.createElement("div");
  div.style.position = "absolute";
  div.style.border = `2px dashed ${drawingColor.value}`;
  div.style.background = `${drawingColor.value}0d`;
  div.style.pointerEvents = "none";
  div.style.zIndex = "10";
  div.style.left = x + "px";
  div.style.top = y + "px";
  div.style.width = "0px";
  div.style.height = "0px";
  el.appendChild(div);
  boxPreviewDiv = div;
}

function updateBoxPreview(x: number, y: number, sx: number, sy: number) {
  if (!boxPreviewDiv) return;
  const left = Math.min(sx, x);
  const top = Math.min(sy, y);
  boxPreviewDiv.style.left = left + "px";
  boxPreviewDiv.style.top = top + "px";
  boxPreviewDiv.style.width = Math.abs(x - sx) + 1 + "px";
  boxPreviewDiv.style.height = Math.abs(y - sy) + 1 + "px";
}

function removeBoxPreview() {
  if (boxPreviewDiv && boxPreviewDiv.parentNode) {
    boxPreviewDiv.parentNode.removeChild(boxPreviewDiv);
    boxPreviewDiv = null;
  }
}

// OCR 文本输入弹窗是否可见
const ocrTextInputVisible = ref(false);
const ocrTextInput = ref("");
const ocrInputRef = ref<InstanceType<typeof NInput> | null>(null);
watch(ocrTextInputVisible, (v) => { if (v) nextTick(() => ocrInputRef.value?.focus()); });
const ocrFirstScreen = ref<{ x: number; y: number } | null>(null);

const ocrModeText = computed(() => {
  if (store.activeTool !== "ocr") return "";
  return store.ocrRectMode ? "□ OCR 矩形模式 — 拖拽绘制，按 T 切换四边形" : "◇ OCR 四边形模式 — 点击放置顶点，点击起始点闭合，按 T 切换矩形";
});

// 关键点阶段2: 矩形框拖拽状态
const kpBoxDrag = reactive({
  active: false,
  startX: 0,
  startY: 0,
});

// ==================== 事件 ====================

// 旋转矩形三步骤专用
const rbStep = ref(0); // 0=未开始, 1=画了点1, 2=画了点2, 3=待画点3
const rbPt1 = ref<{ x: number; y: number } | null>(null);
const rbPt2 = ref<{ x: number; y: number } | null>(null);
const rbDragging = ref(false); // 步骤2：是否在拖拽中（鼠标按下后移动）

// 拖拽/调整状态
const drag = reactive({
  active: false,
  type: "" as "move" | "resize-tl" | "resize-tr" | "resize-bl" | "resize-br" | "resize-l" | "resize-r" | "resize-t" | "resize-b" | "poly-vertex" | "",
  handle: "",
  polyVertexIndex: -1,
  holeIndex: -1, // 孔洞索引（type === "hole-vertex" 时有效）
  ann: null as Annotation | null,
  annOrig: null as any,
  startX: 0,
  startY: 0,
});

// ==================== 样式 ====================

const displayW = computed(() => cw.value * store.zoom);
const displayH = computed(() => ch.value * store.zoom);

const imgStyle = computed(() => ({
  width: `${displayW.value}px`,
  height: `${displayH.value}px`,
  transform: `translate(-50%, -50%) translate(${store.panX}px, ${store.panY}px)`,
}));

const svgStyle = computed(() => ({
  width: `${displayW.value}px`,
  height: `${displayH.value}px`,
  transform: `translate(-50%, -50%) translate(${store.panX}px, ${store.panY}px)`,
}));

// ==================== 鼠标 → 图片像素坐标 ====================

function mouseToImage(e: MouseEvent): { x: number; y: number } | null {
  const el = containerRef.value;
  if (!el || cw.value === 0 || ch.value === 0) return null;
  const rect = el.getBoundingClientRect();
  const dw = displayW.value;
  const dh = displayH.value;
  if (dw === 0 || dh === 0) return null;
  const imgLeft = rect.width / 2 - dw / 2 + store.panX;
  const imgTop = rect.height / 2 - dh / 2 + store.panY;
  const relX = e.clientX - rect.left - imgLeft;
  const relY = e.clientY - rect.top - imgTop;
  return { x: (max(0, min(dw, relX)) / dw) * cw.value, y: (max(0, min(dh, relY)) / dh) * ch.value };
}

/** P1→P2 为矩形的一条完整边；P3 到该边所在直线的垂直距离为邻边长。返回 SVG 模型下的中心与宽高（像素）。 */
function rotatedBoxFromEdgeAndPoint(
  p1: { x: number; y: number },
  p2: { x: number; y: number },
  p3: { x: number; y: number },
): { cx: number; cy: number; width: number; height: number; angle: number } | null {
  const vx = p2.x - p1.x;
  const vy = p2.y - p1.y;
  const width = Math.hypot(vx, vy);
  if (width < 1e-6) return null;

  const mx = (p1.x + p2.x) / 2;
  const my = (p1.y + p2.y) / 2;
  const tx = vx / width;
  const ty = vy / width;

  const t = (p3.x - p1.x) * tx + (p3.y - p1.y) * ty;
  const footx = p1.x + t * tx;
  const footy = p1.y + t * ty;
  const offx = p3.x - footx;
  const offy = p3.y - footy;
  let height = Math.hypot(offx, offy);

  let cx = mx;
  let cy = my;
  if (height >= 1e-6) {
    const nx = offx / height;
    const ny = offy / height;
    cx = mx + (height / 2) * nx;
    cy = my + (height / 2) * ny;
  }

  const angle = Math.atan2(vy, vx);
  return { cx, cy, width, height, angle };
}

// ==================== 事件 ====================

function onImageLoad() {
  if (!imgRef.value) return;
  cw.value = imgRef.value.naturalWidth;
  ch.value = imgRef.value.naturalHeight;
  store.setCanvasSize(cw.value, ch.value);
  // 新图已完全渲染，关闭遮罩并记录
  isLoading.value = false;
  lastLoadedPath.value = store.imagePath ?? "";
}

function onImageError() {
  isLoading.value = false;
  const p = store.imagePath || "";
  store.statusMessage = p
    ? `无法显示图片（路径或权限）: ${p}`
    : "无法显示图片";
}

function onKeyDown(e: KeyboardEvent) {
  // Enter：完成当前绘制
  if (e.key === "Enter") {
    if (store.activeTool === "polygon" && store.drawingPoints.length >= 3) {
      finishPolygonDraw();
    } else if (store.activeTool === "keypoint") {
      if (store.kpPhase === "corners" && store.keypointDrawing.length >= 1) {
        if (props.selectedClassId >= store.classes.length) {
          store.statusMessage = "请先在右侧选择一个类别";
          return;
        }
        store.finishKeypoint(props.selectedClassId);
      } else if (store.kpPhase === "box") {
        if (props.selectedClassId >= store.classes.length) {
          store.statusMessage = "请先在右侧选择一个类别";
          return;
        }
        const boxPreview = store.kpBoxPreview;
        if (boxPreview) {
          store.finishKeypointWithBox(
            props.selectedClassId,
            Math.min(boxPreview.x1, boxPreview.x2),
            Math.min(boxPreview.y1, boxPreview.y2),
            Math.max(boxPreview.x1, boxPreview.x2),
            Math.max(boxPreview.y1, boxPreview.y2)
          );
        }
      }
    } else if (store.activeTool === "ocr" && !store.ocrRectMode && store.ocrDrawingPoints.length >= 3) {
      ocrTextInputVisible.value = true;
    } else if (store.activeTool === "polygon") {
      if (store.drawingPoints.length >= 3) {
        store.finishDraw(props.selectedClassId, 0, 0, 0, 0);
      }
    }
    return;
  }

  // T 键：切换 OCR 矩形/四边形模式
  if ((e.key === "t" || e.key === "T") && store.activeTool === "ocr") {
    store.ocrRectMode = !store.ocrRectMode;
    store.ocrDrawingPoints.length = 0;
    ocrTextInputVisible.value = false;
    ocrFirstScreen.value = null;
    store.statusMessage = store.ocrRectMode ? "OCR: 矩形模式 — 拖拽绘制" : "OCR: 四边形模式 — 点击放置 4 个顶点";
    return;
  }
  // Escape：取消当前绘制
  if (e.key === "Escape") {
    if (store.activeTool === "keypoint") {
      if (store.kpPhase === "box") {
        // 从框阶段退回角点阶段
        store.kpPhase = "corners";
        store.kpBoxPreview = null;
        store.statusMessage = `关键点: 退回角点阶段 (已有 ${store.keypointDrawing.length} 个角点)`;
      } else {
        store.resetKeypointDraw();
      }
    } else if (store.activeTool === "ocr") {
      store.resetOcrDraw();
      ocrTextInputVisible.value = false;
      ocrFirstScreen.value = null;
    } else if (store.activeTool === "polygon") {
      if (holeModeActive.value) {
        holeModeActive.value = false;
        holePolygonId.value = null;
        store.statusMessage = "孔洞模式已退出";
      }
      store.cancelDraw();
    }
  }

  // H 键：切换孔洞模式（多边形工具下）
  if (e.key === "h" || e.key === "H") {
    if (store.activeTool !== "polygon") return;
    holeModeActive.value = !holeModeActive.value;
    if (holeModeActive.value) {
      // 优先用选中的多边形，否则用最后一个
      const selected = store.selectedAnnotation;
      let targetPoly = selected?.type === "Polygon" ? selected : null;
      if (!targetPoly) {
        targetPoly = [...store.annotations].reverse().find(a => a.type === "Polygon") ?? null;
      }
      if (targetPoly) {
        holePolygonId.value = targetPoly.id;
        store.statusMessage = "🕳 孔洞模式已开启！点击并绘制多边形为当前选中标注挖孔，鼠标移至起始点单击完成一个孔洞，按 H 退出";
      } else {
        holeModeActive.value = false;
        store.statusMessage = "没有多边形标注可挖孔，请先画一个多边形";
      }
    } else {
      holePolygonId.value = null;
      store.statusMessage = "孔洞模式已退出";
    }
  }

  // 关键点工具：按 0/1/2 切换待放置角点的可见性
  if (store.activeTool === "keypoint" && store.kpPhase === "corners") {
    if (e.key === "0") { pendingKpVisibility.value = 0; store.statusMessage = `可见性: 隐藏 (v=0)`; e.preventDefault(); }
    else if (e.key === "1") { pendingKpVisibility.value = 1; store.statusMessage = `可见性: 遮挡 (v=1)`; e.preventDefault(); }
    else if (e.key === "2") { pendingKpVisibility.value = 2; store.statusMessage = `可见性: 可见 (v=2)`; e.preventDefault(); }
  }
}

function onWindowMouseUpCapture(e: MouseEvent) {
  finalizeKeypointBoxDrag(e);
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
  // 在画布外松开鼠标时，canvas-container 收不到 mouseup，导致关键点框无法落盘
  window.addEventListener("mouseup", onWindowMouseUpCapture, true);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
  window.removeEventListener("mouseup", onWindowMouseUpCapture, true);
});

function onMouseDown(e: MouseEvent) {
  if (!store.imageLoaded) return;
  // 平移工具
  if (store.activeTool === "pan") {
    drag.active = true;
    drag.type = "move";
    drag.startX = e.clientX;
    drag.startY = e.clientY;
    return;
  }

  const pt = mouseToImage(e);
  if (!pt) return;

  // 矩形工具
  if (store.activeTool === "box") {
    if (props.selectedClassId >= store.classes.length) {
      store.statusMessage = "请先在右侧选择或添加一个类别";
      return;
    }
    document.body.style.cursor = "crosshair";
    showCrosshair.value = true;
    updateCrosshair(e.clientX, e.clientY);
    draw.startX = pt.x;
    draw.startY = pt.y;
    draw.curX = pt.x;
    draw.curY = pt.y;
    const cRect = containerRef.value!.getBoundingClientRect();
    boxStartX.value = e.clientX - cRect.left;
    boxStartY.value = e.clientY - cRect.top;
    startBoxPreview(boxStartX.value, boxStartY.value);
    return;
  }

  // 旋转矩形工具 — 三步绘制
  if (store.activeTool === "rotated_box") {
    document.body.style.cursor = "crosshair";
    if (rbStep.value === 0) {
      // 步骤1：点击画第一个点
      rbPt1.value = { x: pt.x, y: pt.y };
      draw.curX = pt.x;
      draw.curY = pt.y;
      rbStep.value = 1;
    } else if (rbStep.value === 1) {
      // 步骤2：点击确定角度边，进入拖拽模式
      rbPt2.value = { x: pt.x, y: pt.y };
      draw.startX = pt.x;
      draw.startY = pt.y;
      rbDragging.value = true;
      rbStep.value = 2;
    } else if (rbStep.value === 3) {
      // 步骤3：点击确定高度，完成旋转矩形（边 P1P2 + 垂距）
      const p1 = rbPt1.value!;
      const p2 = rbPt2.value!;
      const geom = rotatedBoxFromEdgeAndPoint(p1, p2, pt);
      if (!geom) return;
      store.finishDraw(props.selectedClassId, 0, 0, 0, 0, {
        type: "rotated_box",
        cx: geom.cx / cw.value,
        cy: geom.cy / ch.value,
        width: geom.width / cw.value,
        height: geom.height / ch.value,
        angle: geom.angle,
      });
      rbStep.value = 0;
      rbPt1.value = null;
      rbPt2.value = null;
      draw.curX = null;
      draw.curY = null;
    }
    return;
  }

  // 多边形工具
  if (store.activeTool === "polygon") {
    if (store.drawingPoints.length >= 3 && firstPointScreen.value) {
      const fp = firstPointScreen.value;
      const crect = containerRef.value!.getBoundingClientRect();
      const imgLeft = crect.width / 2 - displayW.value / 2 + store.panX;
      const imgTop = crect.height / 2 - displayH.value / 2 + store.panY;
      const fpClientX = crect.left + imgLeft + fp.x;
      const fpClientY = crect.top + imgTop + fp.y;
      if (Math.hypot(e.clientX - fpClientX, e.clientY - fpClientY) < 20) {
        finishPolygonDraw();
        return;
      }
    }
    store.addDrawPoint(pt.x / cw.value, pt.y / ch.value);
    return;
  }

  // 关键点工具
  if (store.activeTool === "keypoint") {
    // 首次点击或阶段未开始：初始化 kpNames 并进入 corners 阶段（不清空已放置的角点）
    if (store.kpPhase === null) {
      store.startKeypointDraw(props.selectedClassId);
    }
    if (store.kpPhase === "corners") {
      store.addKpCorner(pt.x / cw.value, pt.y / ch.value, pendingKpVisibility.value);
    } else if (store.kpPhase === "box") {
      // 阶段2: 记录拖拽起点（用于实时预览，后续松开完成）
      kpBoxDrag.active = true;
      kpBoxDrag.startX = pt.x / cw.value;
      kpBoxDrag.startY = pt.y / ch.value;
      store.updateKpBoxPreview(kpBoxDrag.startX, kpBoxDrag.startY, kpBoxDrag.startX, kpBoxDrag.startY);
    }
    return;
  }

  // OCR 工具
  if (store.activeTool === "ocr") {
    if (store.ocrRectMode) {
      // 矩形模式 === 完全复刻 box 工具
      if (props.selectedClassId >= store.classes.length) {
        store.statusMessage = "请先在右侧选择或添加一个类别";
        return;
      }
      draw.startX = pt.x;
      draw.startY = pt.y;
      draw.curX = pt.x;
      draw.curY = pt.y;
      const cRect = containerRef.value!.getBoundingClientRect();
      boxStartX.value = e.clientX - cRect.left;
      boxStartY.value = e.clientY - cRect.top;
      startBoxPreview(boxStartX.value, boxStartY.value);
      return;
    } else {
      // 四边形模式：点击放置顶点，点击起始点手动闭合
      if (props.selectedClassId >= store.classes.length) {
        store.statusMessage = "请先在右侧选择或添加一个类别";
        return;
      }
      // 先检查接近闭合（即使已满 4 个点也要能闭合）
      if (store.ocrDrawingPoints.length >= 2 && ocrFirstScreen.value) {
        const fp = ocrFirstScreen.value;
        if (Math.hypot(e.clientX - fp.x, e.clientY - fp.y) < 20) {
          ocrTextInputVisible.value = true;
          return;
        }
      }
      // 再检查最大点数
      if (store.ocrDrawingPoints.length >= 4) return;
      store.addOcrPoint(pt.x / cw.value, pt.y / ch.value);
      if (store.ocrDrawingPoints.length === 1) {
        // 记录第一个点的屏幕坐标
        const crect = containerRef.value!.getBoundingClientRect();
        const dw = displayW.value;
        const imgLeft = crect.width / 2 - dw / 2 + store.panX;
        const scrX = crect.left + imgLeft + store.ocrDrawingPoints[0].x * dw;
        const scrY = crect.top + (crect.height / 2 - displayH.value / 2 + store.panY) + store.ocrDrawingPoints[0].y * displayH.value;
        ocrFirstScreen.value = { x: scrX, y: scrY };
      }
      return;
    }
  }

  // 选择工具：点击空白处取消选择
  if (store.activeTool === "select") {
    store.selectAnnotation(null);
  }
}

function onMouseMove(e: MouseEvent) {
  if (!store.imageLoaded) return;
  const pt = mouseToImage(e);
  store.setCursor(pt ? Math.round(pt.x) : 0, pt ? Math.round(pt.y) : 0);

  // 十字线跟随鼠标
  if (showCrosshair.value) {
    updateCrosshair(e.clientX, e.clientY);
  }

  // 平移
  if (drag.active && drag.type === "move" && store.activeTool === "pan") {
    store.setPan(store.panX + (e.clientX - drag.startX), store.panY + (e.clientY - drag.startY));
    drag.startX = e.clientX;
    drag.startY = e.clientY;
    return;
  }

  // 拖拽/调整标注
    const dx = (e.clientX - drag.startX) / displayW.value;
    const dy = (e.clientY - drag.startY) / displayH.value;

    if (drag.type === "move") {
      if (drag.ann?.type === "AxisAlignedBox") {
        const ann = drag.ann as any;
        const o = drag.annOrig as any;
        ann.x1 = Math.max(0, Math.min(1, o.x1 + dx));
        ann.y1 = Math.max(0, Math.min(1, o.y1 + dy));
        ann.x2 = Math.max(0, Math.min(1, o.x2 + dx));
        ann.y2 = Math.max(0, Math.min(1, o.y2 + dy));
        store.updateAnnotation(ann.id, ann);
      } else if (drag.ann?.type === "RotatedBox") {
        const ann = drag.ann as any;
        const o = drag.annOrig as any;
        ann.cx = Math.max(0, Math.min(1, o.cx + dx));
        ann.cy = Math.max(0, Math.min(1, o.cy + dy));
        store.updateAnnotation(ann.id, ann);
      } else if (drag.ann?.type === "Polygon") {
        const ann = drag.ann as any;
        const o = drag.annOrig as any;
        ann.points = o.points.map((p: any) => ({
          x: Math.max(0, Math.min(1, p.x + dx)),
          y: Math.max(0, Math.min(1, p.y + dy)),
        }));
        store.updateAnnotation(ann.id, ann);
      } else if (drag.ann?.type === "Keypoint") {
        // 点击 kp-bbox 框体拖拽：平移 bounding_box + 所有关键点
        const ann = drag.ann as any;
        const o = drag.annOrig as any;
        const bb = ann.bounding_box;
        const bbO = o.bounding_box;
        if (bb && bbO) {
          bb.cx = Math.max(0, Math.min(1, bbO.cx + dx));
          bb.cy = Math.max(0, Math.min(1, bbO.cy + dy));
          ann.keypoints = o.keypoints.map((kp: any) => {
            const clamped = clampKpToBox(kp.x + dx, kp.y + dy, bb);
            return { ...kp, x: clamped.x, y: clamped.y };
          });
        }
        store.updateAnnotation(ann.id, ann);
      } else if (drag.ann?.type === "Ocr") {
        const ann = drag.ann as any;
        const o = drag.annOrig as any;
        ann.points = o.points.map((p: any) => ({
          x: Math.max(0, Math.min(1, p.x + dx)),
          y: Math.max(0, Math.min(1, p.y + dy)),
        }));
        store.updateAnnotation(ann.id, ann);
      }
      return;
    }

    // ========== 多边形/OCR 顶点拖拽 ==========
    if (drag.type === "poly-vertex" && (drag.ann?.type === "Polygon" || drag.ann?.type === "Ocr")) {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      const idx = drag.polyVertexIndex;
      console.log("[ocr-vertex] idx=", idx, "o=", o.points?.[idx], "dx=", dx, "dy=", dy);
      const newX = Math.max(0, Math.min(1, o.points[idx].x + dx));
      const newY = Math.max(0, Math.min(1, o.points[idx].y + dy));
      ann.points = ann.points.map((p: any, i: number) =>
        i === idx ? { x: newX, y: newY } : p
      );
      store.updateAnnotation(ann.id, ann);
      return;
    }

    // ========== 孔洞顶点拖拽 ==========
    if (drag.type === "hole-vertex" && drag.ann?.type === "Polygon") {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      const hi = drag.holeIndex;
      const pi = drag.polyVertexIndex;
      if (!ann.holes || !ann.holes[hi]) return;
      if (!o.holes || !o.holes[hi]) return;
      const newX = Math.max(0, Math.min(1, o.holes[hi][pi].x + dx));
      const newY = Math.max(0, Math.min(1, o.holes[hi][pi].y + dy));
      ann.holes[hi] = ann.holes[hi].map((p: any, i: number) =>
        i === pi ? { x: newX, y: newY } : p
      );
      store.updateAnnotation(ann.id, ann);
      return;
    }

    // ========== 关键点角点拖拽（约束在框内） ==========
    if (drag.type === "kp-vertex" && drag.ann?.type === "Keypoint") {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      const idx = drag.polyVertexIndex;
      let newX = o.keypoints[idx].x + dx;
      let newY = o.keypoints[idx].y + dy;
      // 约束到 bounding_box 内部
      const bb = ann.bounding_box;
      if (bb) {
        const clamped = clampKpToBox(newX, newY, bb);
        newX = clamped.x;
        newY = clamped.y;
      } else {
        newX = Math.max(0, Math.min(1, newX));
        newY = Math.max(0, Math.min(1, newY));
      }
      ann.keypoints = ann.keypoints.map((p: any, i: number) =>
        i === idx ? { ...p, x: newX, y: newY } : p
      );
      store.updateAnnotation(ann.id, ann);
      return;
    }

    // ========== 关键点包围框平移（框约束关键点） ==========
    if (drag.type === "kp-move" && drag.ann?.type === "Keypoint") {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      const bb = ann.bounding_box;
      const bbO = o.bounding_box;
      if (bb && bbO) {
        const newCx = Math.max(0, Math.min(1, bbO.cx + dx));
        const newCy = Math.max(0, Math.min(1, bbO.cy + dy));
        bb.cx = newCx;
        bb.cy = newCy;
        // 所有关键点同步平移，自动约束在框内
        ann.keypoints = o.keypoints.map((kp: any) => {
          const clamped = clampKpToBox(kp.x + dx, kp.y + dy, bb);
          return { ...kp, x: clamped.x, y: clamped.y };
        });
      }
      store.updateAnnotation(ann.id, ann);
      return;
    }

    // ========== 关键点包围框调整大小（约束关键点在框内，最小框适配关键点） ==========
    if (drag.ann?.type === "Keypoint" && drag.type.startsWith("kp-resize-")) {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      const bb = ann.bounding_box;
      const bbO = o.bounding_box;
      if (!bb || !bbO) return;
      const aspect = ch.value / cw.value;
      const cos = Math.cos(bbO.angle);
      const sin = Math.sin(bbO.angle);
      const h = drag.handle as string;
      // 投影鼠标增量到旋转轴
      const rawW = dx * cos + dy * sin;
      const rawH = -dx * sin + dy * cos;
      const wSign = (h.includes("l") && !h.includes("r")) ? -1
                  : (h.includes("r") && !h.includes("l")) ? 1 : 0;
      const hSign = (h.includes("t") && !h.includes("b")) ? -1
                  : (h.includes("b") && !h.includes("t")) ? 1 : 0;
      bb.width = Math.max(0.001, bbO.width + wSign * rawW);
      bb.height = Math.max(0.001, bbO.height + hSign * rawH);
      // 中心偏移（考虑宽高比）
      const dcx = (bb.width - bbO.width) / 2 * cos * wSign;
      const dcy = (bb.width - bbO.width) / 2 * sin * wSign;
      const dcx2 = -(bb.height - bbO.height) / 2 * sin * hSign;
      const dcy2 = (bb.height - bbO.height) / 2 * cos * hSign;
      bb.cx = Math.max(0, Math.min(1, bbO.cx + dcx + dcx2));
      bb.cy = Math.max(0, Math.min(1, bbO.cy + dcy + dcy2));
      // 所有关键点约束到新的框内
      ann.keypoints = ann.keypoints.map((kp: any) => {
        const clamped = clampKpToBox(kp.x, kp.y, bb);
        return { ...kp, x: clamped.x, y: clamped.y };
      });
      store.updateAnnotation(ann.id, ann);
      return;
    }

    // ========== AxisAlignedBox 调整大小 ==========
    if (drag.ann?.type === "AxisAlignedBox" && drag.type.startsWith("resize-")) {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      const h = drag.handle;
      if (h === "tl") { ann.x1 = Math.max(0, Math.min(o.x2 - 0.001, o.x1 + dx)); ann.y1 = Math.max(0, Math.min(o.y2 - 0.001, o.y1 + dy)); }
      else if (h === "tr") { ann.x2 = Math.max(o.x1 + 0.001, Math.min(1, o.x2 + dx)); ann.y1 = Math.max(0, Math.min(o.y2 - 0.001, o.y1 + dy)); }
      else if (h === "bl") { ann.x1 = Math.max(0, Math.min(o.x2 - 0.001, o.x1 + dx)); ann.y2 = Math.max(o.y1 + 0.001, Math.min(1, o.y2 + dy)); }
      else if (h === "br") { ann.x2 = Math.max(o.x1 + 0.001, Math.min(1, o.x2 + dx)); ann.y2 = Math.max(o.y1 + 0.001, Math.min(1, o.y2 + dy)); }
      else if (h === "l") { ann.x1 = Math.max(0, Math.min(o.x2 - 0.001, o.x1 + dx)); }
      else if (h === "r") { ann.x2 = Math.max(o.x1 + 0.001, Math.min(1, o.x2 + dx)); }
      else if (h === "t") { ann.y1 = Math.max(0, Math.min(o.y2 - 0.001, o.y1 + dy)); }
      else if (h === "b") { ann.y2 = Math.max(o.y1 + 0.001, Math.min(1, o.y2 + dy)); }
      store.updateAnnotation(ann.id, ann);
      return;
    }

    // ========== OCR 框调整大小 ==========
    if (drag.ann?.type === "Ocr" && drag.type.startsWith("resize-")) {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      const h = drag.handle;
      if (!o.points || o.points.length < 4) return;
      // 计算原包围盒
      const xs = o.points.map((p: any) => p.x);
      const ys = o.points.map((p: any) => p.y);
      const bb = { x1: Math.min(...xs), x2: Math.max(...xs), y1: Math.min(...ys), y2: Math.max(...ys) };
      const bw = bb.x2 - bb.x1;
      const bh = bb.y2 - bb.y1;
      // 新包围盒
      let nbb = { ...bb };
      if (h === "tl") { nbb.x1 = Math.max(0, Math.min(nbb.x2 - 0.001, bb.x1 + dx)); nbb.y1 = Math.max(0, Math.min(nbb.y2 - 0.001, bb.y1 + dy)); }
      else if (h === "tr") { nbb.x2 = Math.max(nbb.x1 + 0.001, Math.min(1, bb.x2 + dx)); nbb.y1 = Math.max(0, Math.min(nbb.y2 - 0.001, bb.y1 + dy)); }
      else if (h === "bl") { nbb.x1 = Math.max(0, Math.min(nbb.x2 - 0.001, bb.x1 + dx)); nbb.y2 = Math.max(nbb.y1 + 0.001, Math.min(1, bb.y2 + dy)); }
      else if (h === "br") { nbb.x2 = Math.max(nbb.x1 + 0.001, Math.min(1, bb.x2 + dx)); nbb.y2 = Math.max(nbb.y1 + 0.001, Math.min(1, bb.y2 + dy)); }
      const nw = nbb.x2 - nbb.x1;
      const nh = nbb.y2 - nbb.y1;
      if (nw < 0.001 || nh < 0.001) return;
      // 将原 4 个点按比例映射到新包围盒
      ann.points = o.points.map((p: any) => ({
        x: nbb.x1 + (p.x - bb.x1) / bw * nw,
        y: nbb.y1 + (p.y - bb.y1) / bh * nh,
      }));
      store.updateAnnotation(ann.id, ann);
      return;
    }
    if (drag.ann?.type === "RotatedBox" && drag.type.startsWith("resize-")) {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      const h = drag.handle;
      const cos = Math.cos(o.angle);
      const sin = Math.sin(o.angle);
      const aspect = ch.value / cw.value;
      if (!pt) return;
      const mx = pt.x / cw.value, my = pt.y / ch.value;
      let fx = 0, fy = 0;
      if (h === "tl") { fx = 1; fy = 1; } else if (h === "tr") { fx = -1; fy = 1; }
      else if (h === "bl") { fx = 1; fy = -1; } else if (h === "br") { fx = -1; fy = -1; }
      const fix_x = o.cx + fx * o.width / 2 * cos - fy * o.height / 2 * aspect * sin;
      const fix_y = o.cy + fx * o.width / 2 / aspect * sin + fy * o.height / 2 * cos;
      // 鼠标位置 = 新角位置
      const newCx = (fix_x + mx) / 2;
      const newCy = (fix_y + my) / 2;
      // 从中心到新角在 viewBox 中的偏移，再反旋转得到局部宽高
      const dvx = (mx - newCx) * cw.value;
      const dvy = (my - newCy) * ch.value;
      const lx = dvx * cos + dvy * sin;
      const ly = -dvx * sin + dvy * cos;
      ann.width = Math.max(0.001, Math.abs(lx) * 2 / cw.value);
      ann.height = Math.max(0.001, Math.abs(ly) * 2 / ch.value);
      ann.cx = Math.max(0, Math.min(1, newCx));
      ann.cy = Math.max(0, Math.min(1, newCy));
      store.updateAnnotation(ann.id, ann);
      return;
    }

    // ========== RotatedBox 旋转 ==========
    if (drag.ann?.type === "RotatedBox" && drag.type === "rotate") {
      const ann = drag.ann as any;
      const o = drag.annOrig as any;
      // 计算标注中心在屏幕上的坐标
      const el = containerRef.value;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      const dw = displayW.value;
      const dh = displayH.value;
      const imgLeft = rect.width / 2 - dw / 2 + store.panX;
      const imgTop = rect.height / 2 - dh / 2 + store.panY;
      const cx = rect.left + imgLeft + o.cx * dw;
      const cy = rect.top + imgTop + o.cy * dh;
      const prevAngle = Math.atan2(drag.startY - cy, drag.startX - cx);
      const curAngle = Math.atan2(e.clientY - cy, e.clientX - cx);
      ann.angle = o.angle + (curAngle - prevAngle);
      store.updateAnnotation(ann.id, ann);
      return;
    }

  // 旋转矩形步骤2：拖拽中实时更新
  if (pt && store.activeTool === "rotated_box" && rbStep.value === 2) {
    if (rbDragging.value) {
      rbPt2.value = { x: pt.x, y: pt.y };
    } else if (draw.startX !== null && draw.startY !== null) {
      const dist = Math.hypot(pt.x - draw.startX, pt.y - draw.startY);
      if (dist > 5) {
        rbDragging.value = true;
        rbPt2.value = { x: pt.x, y: pt.y };
      }
    }
    return;
  }

  // 步骤1 / 步骤3：更新跟随线
  if (pt && store.activeTool === "rotated_box" && (rbStep.value === 1 || rbStep.value === 3)) {
    draw.curX = pt.x;
    draw.curY = pt.y;
    return;
  }

  // 关键点阶段2: 矩形框跟随鼠标实时预览
  if (pt && store.activeTool === "keypoint" && store.kpPhase === "box" && kpBoxDrag.active) {
    const curX = pt.x / cw.value;
    const curY = pt.y / ch.value;
    store.updateKpBoxPreview(kpBoxDrag.startX, kpBoxDrag.startY, curX, curY);
    draw.curX = pt.x;
    draw.curY = pt.y;
    return;
  }

  // 矩形工具：拖拽中实时预览
  if (store.activeTool === "box" && draw.startX !== null && draw.startY !== null) {
    updateCrosshair(e.clientX, e.clientY);
    const cRect = containerRef.value!.getBoundingClientRect();
    const curX = e.clientX - cRect.left;
    const curY = e.clientY - cRect.top;
    updateBoxPreview(curX, curY, boxStartX.value, boxStartY.value);
    if (pt) {
      draw.curX = pt.x;
      draw.curY = pt.y;
    }
    return;
  }

  // OCR 矩形模式：实时预览
  if (store.activeTool === "ocr" && store.ocrRectMode && draw.startX !== null && draw.startY !== null) {
    const cRect = containerRef.value!.getBoundingClientRect();
    const curX = e.clientX - cRect.left;
    const curY = e.clientY - cRect.top;
    updateBoxPreview(curX, curY, boxStartX.value, boxStartY.value);
    if (pt) {
      draw.curX = pt.x;
      draw.curY = pt.y;
    }
    return;
  }

  if (pt) {
    draw.curX = pt.x;
    draw.curY = pt.y;
  }
}

/**
 * 关键点阶段2：在任意位置松开鼠标都应收尾（依赖 window capture，避免仅在画布内 mouseup 才生效）
 */
function finalizeKeypointBoxDrag(e: MouseEvent) {
  if (!store.imageLoaded) { console.log("[DEBUG finalize] no image"); return; }
  if (store.activeTool !== "keypoint") { console.log("[DEBUG finalize] not keypoint tool, tool=", store.activeTool); return; }
  if (store.kpPhase !== "box") { console.log("[DEBUG finalize] not box phase, kpPhase=", store.kpPhase); return; }
  if (!kpBoxDrag.active) { console.log("[DEBUG finalize] kpBoxDrag not active"); return; }

  const pt = mouseToImage(e);
  if (pt) {
    const x1 = Math.min(kpBoxDrag.startX, pt.x / cw.value);
    const y1 = Math.min(kpBoxDrag.startY, pt.y / ch.value);
    const x2 = Math.max(kpBoxDrag.startX, pt.x / cw.value);
    const y2 = Math.max(kpBoxDrag.startY, pt.y / ch.value);
    store.updateKpBoxPreview(x1, y1, x2, y2);
  }
  // 松开在侧栏/窗口外时 mouseToImage 为 null，沿用拖拽过程中最后的预览框
  kpBoxDrag.active = false;
  kpBoxDrag.startX = 0;
  kpBoxDrag.startY = 0;

  // ---- 防止单击即完成：检查实际拖拽距离 ----
  const boxPreview = store.kpBoxPreview ? { ...store.kpBoxPreview } : null;
  const dist = boxPreview
    ? Math.hypot((boxPreview.x2 - boxPreview.x1) * cw.value, (boxPreview.y2 - boxPreview.y1) * ch.value)
    : 0;
  if (dist < 8) {
    store.kpBoxPreview = null;
    kpBoxDrag.active = false;
    return;
  }

  if (store.keypointDrawing.length >= 1 && boxPreview) {
    store.finishKeypointWithBox(
      props.selectedClassId,
      Math.min(boxPreview.x1, boxPreview.x2),
      Math.min(boxPreview.y1, boxPreview.y2),
      Math.max(boxPreview.x1, boxPreview.x2),
      Math.max(boxPreview.y1, boxPreview.y2)
    );
  }
}

function onMouseUp(e: MouseEvent) {
  document.body.style.cursor = "";
  if (store.activeTool === "select" || store.activeTool === "pan") {
    showCrosshair.value = false;
  }
  if (!store.imageLoaded) return;

  if (drag.active && store.activeTool === "pan") {
    drag.active = false;
    drag.type = "";
    return;
  }
  if (drag.active && (drag.type === "move" || drag.type === "poly-vertex" || drag.type === "hole-vertex" || drag.type === "rotate" || drag.type.startsWith("resize-") || drag.type.startsWith("kp-"))) {
    if (drag.ann) {
      store.updateAnnotation(drag.ann.id, drag.ann as any);
    }
    drag.active = false;
    drag.type = "";
    drag.handle = "";
    drag.ann = null;
    drag.annOrig = null;
    drag.polyVertexIndex = -1;
    drag.holeIndex = -1;
    return;
  }

  // 旋转矩形步骤2结束
  if (store.activeTool === "rotated_box" && rbStep.value === 2) {
    if (!rbDragging.value) {
      // 快速点击，直接进入步骤3
      rbStep.value = 3;
    } else {
      // 拖拽后松开，确认点2，进入步骤3
      rbPt2.value = { x: draw.curX ?? draw.startX!, y: draw.curY ?? draw.startY! };
      rbStep.value = 3;
      rbDragging.value = false;
    }
    draw.startX = null;
    draw.startY = null;
    return;
  }

  // 关键点框拖拽收尾由 window capture 的 finalizeKeypointBoxDrag 统一处理（此处不再重复）

  // 矩形完成
  if (store.activeTool === "box" && draw.startX !== null && draw.startY !== null) {
    const pt = mouseToImage(e);
    if (pt && draw.curX !== null && draw.curY !== null) {
      const w = Math.abs(pt.x - draw.startX);
      const h = Math.abs(pt.y - draw.startY);
      if (w > 5 || h > 5) {
        store.finishDraw(props.selectedClassId, draw.startX / cw.value, draw.startY / ch.value, pt.x / cw.value, pt.y / ch.value);
      }
    }
    draw.startX = null;
    draw.startY = null;
    draw.curX = null;
    draw.curY = null;
    removeBoxPreview();
  }

  // OCR 矩形模式完成
  if (store.activeTool === "ocr" && store.ocrRectMode && draw.startX !== null && draw.startY !== null) {
    const pt = mouseToImage(e);
    if (pt && draw.curX !== null && draw.curY !== null) {
      const w = Math.abs(pt.x - draw.startX);
      const h = Math.abs(pt.y - draw.startY);
      if (w > 5 || h > 5) {
        store.ocrDrawingPoints.length = 0;
        const x1 = Math.min(draw.startX, pt.x) / cw.value;
        const y1 = Math.min(draw.startY, pt.y) / ch.value;
        const x2 = Math.max(draw.startX, pt.x) / cw.value;
        const y2 = Math.max(draw.startY, pt.y) / ch.value;
        store.ocrDrawingPoints.push({ x: x1, y: y1 }, { x: x2, y: y1 }, { x: x2, y: y2 }, { x: x1, y: y2 });
        ocrTextInputVisible.value = true;
      }
    }
    draw.startX = null;
    draw.startY = null;
    draw.curX = null;
    draw.curY = null;
    removeBoxPreview();
  }
}

function confirmOcrText() {
  if (ocrTextInput.value.trim()) {
    store.setOcrText(ocrTextInput.value);
    store.finishOcr(props.selectedClassId);
    ocrTextInput.value = "";
    ocrTextInputVisible.value = false;
  }
}

function onMouseEnter(e: MouseEvent) {
  if (store.activeTool !== "select" && store.activeTool !== "pan") {
    showCrosshair.value = true;
    updateCrosshair(e.clientX, e.clientY);
  }
}

function onMouseLeave() {
  document.body.style.cursor = "";
  showCrosshair.value = false;
  draw.curX = null;
  draw.curY = null;
  removeBoxPreview();
  if (drag.active && drag.ann && store.activeTool === "select") {
    if (drag.type === "poly-vertex") {
      store.updateAnnotation(drag.ann.id, drag.ann as any);
    }
    drag.active = false;
    drag.type = "";
    drag.handle = "";
    drag.polyVertexIndex = -1;
    drag.ann = null;
    drag.annOrig = null;
  }
}

function onWheel(e: WheelEvent) {
  if (!store.imageLoaded) return;
  store.setZoom(store.zoom + (e.deltaY > 0 ? -0.08 : 0.08));
}

function finishPolygonDraw() {
  if (holeModeActive.value && holePolygonId.value) {
    const ann = store.annotations.find(a => a.id === holePolygonId.value) as any;
    if (ann && ann.holes !== undefined) {
      ann.holes.push(store.drawingPoints.map(p => ({ x: p.x, y: p.y })));
      store.updateAnnotation(ann.id, ann);
      // 校验孔洞
      const check = validatePolygonHoles(ann);
      if (!check.valid) {
        store.statusMessage = "⚠️ " + check.errors[0];
      } else {
        store.statusMessage = "孔洞已添加。继续画下一个孔洞，或按 H 退出孔洞模式";
      }
    }
    store.statusMessage = "孔洞已添加。继续画下一个孔洞，或按 H 退出孔洞模式";
  } else {
    store.finishDraw(props.selectedClassId, 0, 0, 0, 0);
    store.statusMessage = "多边形已创建";
  }
  draw.curX = null;
  draw.curY = null;
}

function onDblClick(_e: MouseEvent) {
  if (store.activeTool === "polygon" && store.drawingPoints.length >= 3) {
    finishPolygonDraw();
  }
}

// ==================== 标注交互 ====================

function onAnnotationMousedown(e: MouseEvent, ann: Annotation) {
  if (store.activeTool !== "select") return;
  e.stopPropagation();

  const el = e.target as Element;
  const currentEl = e.currentTarget as Element;
  const tag = el.tagName.toLowerCase();
  const dh = el.getAttribute("data-handle");
  const kpAnnId = el.getAttribute("data-kp-ann-id");
  console.log("[ann-click] tag=", tag, "dh=", dh, "type=", ann.type, "kpAnnId=", kpAnnId);

  // ==================== 带 data-handle 的控制元素优先处理 ====================

  // 关键点角点拖拽
  if (dh && dh.startsWith("kp-") && kpAnnId) {
    const idx = parseInt(dh.replace("kp-", ""), 10);
    if (!isNaN(idx)) {
      store.selectAnnotation(ann.id);
      drag.active = true;
      drag.type = "kp-vertex";
      drag.polyVertexIndex = idx;
      drag.ann = ann;
      drag.annOrig = cloneAnnotation(ann);
      drag.startX = e.clientX;
      drag.startY = e.clientY;
      return;
    }
  }

  // 关键点包围框手柄（tl/tr/... + data-kp-ann-id）
  if (dh && dh !== "move" && kpAnnId && ann.type === "Keypoint") {
    store.selectAnnotation(ann.id);
    drag.active = true;
    drag.type = ("kp-resize-" + dh) as any;
    drag.handle = dh;
    drag.ann = ann;
    drag.annOrig = cloneAnnotation(ann);
    drag.startX = e.clientX;
    drag.startY = e.clientY;
    return;
  }

  // 关键点包围框移动手柄
  if (dh === "move" && kpAnnId && ann.type === "Keypoint") {
    store.selectAnnotation(ann.id);
    drag.active = true;
    drag.type = "kp-move";
    drag.ann = ann;
    drag.annOrig = cloneAnnotation(ann);
    drag.startX = e.clientX;
    drag.startY = e.clientY;
    return;
  }

  // 孔洞顶点拖拽
  if (dh && /^h\d+-\d+$/.test(dh)) {
    const parts = dh.split("-");
    const hi = parseInt(parts[0].substring(1), 10);
    const pi = parseInt(parts[1], 10);
    if (!isNaN(hi) && !isNaN(pi)) {
      store.selectAnnotation(ann.id);
      drag.active = true;
      drag.type = "hole-vertex";
      drag.holeIndex = hi;
      drag.polyVertexIndex = pi;
      drag.ann = ann;
      drag.annOrig = cloneAnnotation(ann);
      drag.startX = e.clientX;
      drag.startY = e.clientY;
      return;
    }
  }

  // OCR 四边形顶点拖拽
  if (dh && dh.startsWith("ocr-vertex-")) {
    const idx = parseInt(dh.replace("ocr-vertex-", ""), 10);
    if (!isNaN(idx) && ann.type === "Ocr") {
      store.selectAnnotation(ann.id);
      drag.active = true;
      drag.type = "poly-vertex";
      drag.polyVertexIndex = idx;
      drag.ann = ann;
      drag.annOrig = cloneAnnotation(ann);
      drag.startX = e.clientX;
      drag.startY = e.clientY;
      return;
    }
  }

  // OCR 包围框缩放
  if (dh && dh.startsWith("ocr-resize-")) {
    const handle = dh.replace("ocr-resize-", "");
    store.selectAnnotation(ann.id);
    drag.active = true;
    drag.type = ("resize-" + handle) as any;
    drag.handle = handle;
    drag.ann = ann;
    drag.annOrig = cloneAnnotation(ann);
    drag.startX = e.clientX;
    drag.startY = e.clientY;
    return;
  }

  // 多边形顶点拖拽
  if (dh && dh.startsWith("poly-")) {
    const idx = parseInt(dh.replace("poly-", ""), 10);
    if (!isNaN(idx)) {
      store.selectAnnotation(ann.id);
      drag.active = true;
      drag.type = "poly-vertex";
      drag.polyVertexIndex = idx;
      drag.ann = ann;
      drag.annOrig = cloneAnnotation(ann);
      drag.startX = e.clientX;
      drag.startY = e.clientY;
      return;
    }
  }

  // ==================== 按 tag 类型分发 ====================

  if (tag === "polygon" || tag === "path") {
    // 点击多边形框体 → 拖拽移动
    store.selectAnnotation(ann.id);
    drag.active = true;
    drag.type = "move";
    drag.ann = ann;
    drag.annOrig = cloneAnnotation(ann);
    drag.startX = e.clientX;
    drag.startY = e.clientY;
    return;
  }

  if (tag === "rect") {
    // data-handle="move" → 旋转矩形/关键点框中心手柄，拖拽移动
    if (dh === "move") {
      store.selectAnnotation(ann.id);
      drag.active = true;
      drag.type = "move";
      drag.ann = ann;
      drag.annOrig = cloneAnnotation(ann);
      drag.startX = e.clientX;
      drag.startY = e.clientY;
      return;
    }

    // 有 data-handle（tl/tr/...）→ 边/角控制点，调整大小
    if (dh && dh !== "move" && dh !== "rotate") {
      document.body.style.cursor = "";
      store.selectAnnotation(ann.id);
      drag.active = true;
      drag.type = ("resize-" + dh) as any;
      drag.handle = dh;
      drag.ann = ann;
      drag.annOrig = cloneAnnotation(ann);
      drag.startX = e.clientX;
      drag.startY = e.clientY;
      return;
    }

    // 无 data-handle 的 rect → 主矩形框体（AxisAlignedBox / RotatedBox / Keypoint kp-bbox）
    store.selectAnnotation(ann.id);
    drag.active = true;
    drag.type = "move";
    drag.ann = ann;
    drag.annOrig = cloneAnnotation(ann);
    drag.startX = e.clientX;
    drag.startY = e.clientY;
    return;
  }

  // circle 元素（旋转手柄）
  if (tag === "circle" && dh === "rotate") {
    document.body.style.cursor = "";
    store.selectAnnotation(ann.id);
    drag.active = true;
    drag.type = "rotate";
    drag.ann = ann;
    drag.annOrig = cloneAnnotation(ann);
    drag.startX = e.clientX;
    drag.startY = e.clientY;
    return;
  }

  // 点击其他区域 → 仅选中
  store.selectAnnotation(ann.id);
}

function cloneAnnotation(ann: Annotation): any {
  if (ann.type === "AxisAlignedBox") return { x1: (ann as any).x1, y1: (ann as any).y1, x2: (ann as any).x2, y2: (ann as any).y2 };
  if (ann.type === "RotatedBox") return { cx: (ann as any).cx, cy: (ann as any).cy, width: (ann as any).width, height: (ann as any).height, angle: (ann as any).angle };
  if (ann.type === "Polygon") return { points: (ann as any).points.map((p: any) => ({ ...p })), holes: (ann as any).holes?.map((h: any[]) => h.map((p: any) => ({ ...p }))) };
  if (ann.type === "Keypoint") {
    return {
      bounding_box: (ann as any).bounding_box ? { ...(ann as any).bounding_box } : null,
      keypoints: (ann as any).keypoints.map((kp: any) => ({ ...kp })),
    };
  }
  if (ann.type === "Ocr") return { points: (ann as any).points.map((p: any) => ({ ...p })) };
  return {};
}

// ==================== 多边形辅助 ====================

const firstPointScreen = computed(() => {
  if (store.drawingPoints.length === 0) return null;
  const p = store.drawingPoints[0];
  return { x: p.x * displayW.value, y: p.y * displayH.value };
});

// ==================== OCR 辅助 ====================

const ocrFollowLine = computed(() => {
  if (store.activeTool !== "ocr" || store.ocrRectMode || store.ocrDrawingPoints.length === 0 || draw.curX === null || draw.curY === null) return null;
  const last = store.ocrDrawingPoints[store.ocrDrawingPoints.length - 1];
  return { x1: last.x * cw.value, y1: last.y * ch.value, x2: draw.curX, y2: draw.curY };
});

const ocrNearFirst = computed(() => {
  if (store.ocrRectMode || store.ocrDrawingPoints.length < 2) return false;
  if (draw.curX === null || draw.curY === null) return false;
  const fx = store.ocrDrawingPoints[0].x * cw.value;
  const fy = store.ocrDrawingPoints[0].y * ch.value;
  const dx = draw.curX - fx;
  const dy = draw.curY - fy;
  return Math.sqrt(dx * dx + dy * dy) < 30;
});

const followLineEndScreen = computed(() => {
  if (draw.curX === null || draw.curY === null) return null;
  return {
    x: (draw.curX / cw.value) * displayW.value,
    y: (draw.curY / ch.value) * displayH.value,
  };
});

const polylineNearFirst = computed(() => {
  if (store.drawingPoints.length < 3) return false;
  if (!firstPointScreen.value || !followLineEndScreen.value) return false;
  const dx = firstPointScreen.value.x - followLineEndScreen.value.x;
  const dy = firstPointScreen.value.y - followLineEndScreen.value.y;
  return Math.sqrt(dx * dx + dy * dy) < 20;
});

// ==================== 预览 ====================

// 第3步：垂足 → 光标，表示「到第一条边的垂直距离」
const rbStep3PerpLine = computed(() => {
  if (store.activeTool !== "rotated_box" || rbStep.value !== 3 || !rbPt1.value || !rbPt2.value) return null;
  if (draw.curX === null || draw.curY === null) return null;
  const p1 = rbPt1.value;
  const p2 = rbPt2.value;
  const p3 = { x: draw.curX, y: draw.curY };
  const vx = p2.x - p1.x;
  const vy = p2.y - p1.y;
  const w = Math.hypot(vx, vy);
  if (w < 1e-6) return null;
  const tx = vx / w;
  const ty = vy / w;
  const t = (p3.x - p1.x) * tx + (p3.y - p1.y) * ty;
  return {
    x1: p1.x + t * tx,
    y1: p1.y + t * ty,
    x2: p3.x,
    y2: p3.y,
  };
});

// 旋转矩形预览（第3步实时显示矩形）
const rbFinalPreview = computed(() => {
  if (store.activeTool !== "rotated_box" || rbStep.value !== 3 || !rbPt1.value || !rbPt2.value) return null;
  if (draw.curX === null || draw.curY === null) return null;

  const geom = rotatedBoxFromEdgeAndPoint(rbPt1.value, rbPt2.value, { x: draw.curX, y: draw.curY });
  if (!geom) return null;

  const angleDeg = (geom.angle * 180) / Math.PI;
  return {
    cx: geom.cx,
    cy: geom.cy,
    x: geom.cx - geom.width / 2,
    y: geom.cy - geom.height / 2,
    w: geom.width,
    h: geom.height,
    angle: angleDeg,
  };
});

const polyFollowLine = computed(() => {
  if (store.activeTool !== "polygon" || store.drawingPoints.length === 0 || draw.curX === null || draw.curY === null) return null;
  const last = store.drawingPoints[store.drawingPoints.length - 1];
  return { x1: last.x * cw.value, y1: last.y * ch.value, x2: draw.curX, y2: draw.curY! };
});

// ==================== 分类标注 ====================

const selectedClassificationIds = computed(() => {
  const cls = store.annotations.find((a) => a.type === "Classification") as ClassificationAnnotation | undefined;
  return cls ? cls.class_ids : [];
});

// ==================== 辅助 ====================

function getClassColor(classId: number): string {
  return store.classes[classId]?.color ?? "#ffffff";
}

const drawingColor = computed(() => getClassColor(props.selectedClassId));

function lightenColor(hex: string, amount: number): string {
  const num = parseInt(hex.replace("#", ""), 16);
  const r = Math.min(255, ((num >> 16) & 0xff) + amount);
  const g = Math.min(255, ((num >> 8) & 0xff) + amount);
  const b = Math.min(255, (num & 0xff) + amount);
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, "0")}`;
}

function getClassName(classId: number): string {
  return store.classes[classId]?.name ?? `Class ${classId}`;
}

/** 顶边类别标签高度、控制点（与参考 VisionLabel 接近的细线 + 小白方） */
const LABEL_TAG_H = 14;
const H_SZ = 6;
const H_HALF = 3;

function labelWidthForClass(classId: number): number {
  const name = getClassName(classId);
  return Math.max(26, textPixelWidth(name) + 8);
}

function textPixelWidth(text: string): number {
  let w = 0;
  for (const ch of text) {
    w += /[\u4e00-\u9fff\u3400-\u4dbf\uf900-\ufaff]/.test(ch) ? 10 : 5.5;
  }
  return Math.round(w);
}

function aabLabelLayout(ann: AxisAlignedBox) {
  const w = labelWidthForClass(ann.class_id);
  const left = ann.x1 * cw.value;
  const top = ann.y1 * ch.value;
  return {
    x: left,
    y: top - LABEL_TAG_H,
    w,
    tx: left + w / 2,
    ty: top - LABEL_TAG_H / 2,
  };
}

function rotatedBoxTopLeft(ann: RotatedBox): { x: number; y: number } {
  const hw = (ann.width * cw.value) / 2;
  const hh = (ann.height * ch.value) / 2;
  const cos = Math.cos(ann.angle);
  const sin = Math.sin(ann.angle);
  return {
    x: ann.cx * cw.value + (-hw) * cos - (-hh) * sin,
    y: ann.cy * ch.value + (-hw) * sin + (-hh) * cos,
  };
}

function rbLabelLayout(ann: RotatedBox) {
  const w = labelWidthForClass(ann.class_id);
  const tl = rotatedBoxTopLeft(ann);
  return {
    x: tl.x,
    y: tl.y - LABEL_TAG_H,
    w,
    tx: tl.x + w / 2,
    ty: tl.y - LABEL_TAG_H / 2,
  };
}

function polygonTopLeft(ann: PolygonAnnotation): { x: number; y: number } {
  const pts = ann.points.map((p) => ({ x: p.x * cw.value, y: p.y * ch.value }));
  let minY = Infinity;
  let leftX = 0;
  for (const p of pts) {
    if (p.y < minY) { minY = p.y; leftX = p.x; }
    if (p.y === minY && p.x < leftX) { leftX = p.x; }
  }
  return { x: leftX, y: minY };
}

function polyLabelLayout(ann: PolygonAnnotation) {
  const w = labelWidthForClass(ann.class_id);
  const tl = polygonTopLeft(ann);
  return {
    x: tl.x,
    y: tl.y - LABEL_TAG_H,
    w,
    tx: tl.x + w / 2,
    ty: tl.y - LABEL_TAG_H / 2,
  };
}

function polyBBox(ann: PolygonAnnotation) {
  const W = cw.value, H = ch.value;
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  function addPts(pts: Point[]) {
    for (const p of pts) {
      const px = p.x * W, py = p.y * H;
      if (px < minX) minX = px;
      if (py < minY) minY = py;
      if (px > maxX) maxX = px;
      if (py > maxY) maxY = py;
    }
  }
  addPts(ann.points);
  if (ann.holes) for (const hole of ann.holes) addPts(hole);
  return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
}

function polyBBoxLabel(ann: PolygonAnnotation, bbox: { x: number; y: number; w: number; h: number }) {
  const w = labelWidthForClass(ann.class_id);
  return {
    x: bbox.x,
    y: bbox.y - LABEL_TAG_H,
    w,
    tx: bbox.x + w / 2,
    ty: bbox.y - LABEL_TAG_H / 2,
  };
}

function polygonPath(ann: PolygonAnnotation): string {
  const W = cw.value, H = ch.value;
  function ptsToPath(pts: Point[]) {
    return pts.map((p, i) => `${i === 0 ? 'M' : 'L'}${p.x * W},${p.y * H}`).join('') + 'Z';
  }
  let d = ptsToPath(ann.points);
  if (ann.holes) {
    for (const hole of ann.holes) {
      if (hole.length >= 3) d += ptsToPath(hole);
    }
  }
  return d;
}

/** 射线法判断点是否在多边形内 */
function pointInPolygon(px: number, py: number, polygon: Point[]): boolean {
  let inside = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const xi = polygon[i].x, yi = polygon[i].y;
    const xj = polygon[j].x, yj = polygon[j].y;
    if ((yi > py) !== (yj > py) && px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
      inside = !inside;
    }
  }
  return inside;
}

/** 验证多边形孔洞是否合法：所有孔必须在外部多边形内部，且不超出边界 */
function validatePolygonHoles(ann: PolygonAnnotation): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  if (!ann.holes || ann.holes.length === 0) return { valid: true, errors };
  for (let hi = 0; hi < ann.holes.length; hi++) {
    const hole = ann.holes[hi];
    if (!hole || hole.length < 3) {
      errors.push(`孔洞 #${hi + 1} 点数不足`);
      continue;
    }
    // 检查孔洞的每个顶点是否在外多边形内部
    for (let pi = 0; pi < hole.length; pi++) {
      const p = hole[pi];
      if (!pointInPolygon(p.x, p.y, ann.points)) {
        errors.push(`孔洞 #${hi + 1} 的顶点 ${pi + 1} (${p.x.toFixed(3)}, ${p.y.toFixed(3)}) 超出了外部多边形边界`);
        break;
      }
    }
  }
  return { valid: errors.length === 0, errors };
}

function aabEdgeHandles(ann: AxisAlignedBox) {
  const W = cw.value;
  const H = ch.value;
  const mx = ((ann.x1 + ann.x2) / 2) * W;
  const my = ((ann.y1 + ann.y2) / 2) * H;
  return [
    { h: "t" as const, x: mx, y: ann.y1 * H },
    { h: "b" as const, x: mx, y: ann.y2 * H },
    { h: "l" as const, x: ann.x1 * W, y: my },
    { h: "r" as const, x: ann.x2 * W, y: my },
  ];
}

function aabCornerHandles(ann: AxisAlignedBox) {
  const W = cw.value;
  const H = ch.value;
  return [
    { h: "tl" as const, x: ann.x1 * W, y: ann.y1 * H },
    { h: "tr" as const, x: ann.x2 * W, y: ann.y1 * H },
    { h: "bl" as const, x: ann.x1 * W, y: ann.y2 * H },
    { h: "br" as const, x: ann.x2 * W, y: ann.y2 * H },
  ];
}

/** 计算旋转矩形某个手柄在 SVG 像素坐标下的位置。
 * ann: RotatedBox（归一化），h: tl|tc|tr|mr|br|bc|bl|ml，cw/ch: SVG 像素宽高。
 * 返回 {x, y} 单位为 SVG 像素。 */
function rbHandlePos(ann: any, h: string, cw: number, ch: number): { x: number; y: number } {
  const cos = Math.cos(ann.angle);
  const sin = Math.sin(ann.angle);
  const hw = (ann.width * cw) / 2;
  const hh = (ann.height * ch) / 2;
  let lx = 0, ly = 0;
  if (h === "tl") { lx = -hw; ly = -hh; }
  else if (h === "tc") { lx = 0; ly = -hh; }
  else if (h === "tr") { lx = hw; ly = -hh; }
  else if (h === "mr") { lx = hw; ly = 0; }
  else if (h === "br") { lx = hw; ly = hh; }
  else if (h === "bc") { lx = 0; ly = hh; }
  else if (h === "bl") { lx = -hw; ly = hh; }
  else if (h === "ml") { lx = -hw; ly = 0; }
  return {
    x: ann.cx * cw + lx * cos - ly * sin,
    y: ann.cy * ch + lx * sin + ly * cos,
  };
}

function rbRotateHandlePos(ann: any, cw: number, ch: number): { x: number; y: number } {
  const cos = Math.cos(ann.angle);
  const sin = Math.sin(ann.angle);
  const hh = (ann.height * ch) / 2;
  const offset = 25;
  const lx = 0;
  const ly = -hh - offset;
  return {
    x: ann.cx * cw + lx * cos - ly * sin,
    y: ann.cy * ch + lx * sin + ly * cos,
  };
}

// ==================== 关键点辅助 ====================

/** 计算关键点标注的旋转包围框手柄位置（与 rbHandlePos 相同接口） */
function kpRbHandlePos(bb: any, h: string, cw: number, ch: number): { x: number; y: number } {
  return rbHandlePos(bb, h, cw, ch);
}

/** 检查关键点 (x, y) 是否在旋转包围框 bb（归一化坐标）内部（考虑 viewBox 宽高比） */
function kpInsideBox(x: number, y: number, bb: any): boolean {
  if (!bb) return false;
  const aspect = ch.value / cw.value;
  const cos = Math.cos(bb.angle);
  const sin = Math.sin(bb.angle);
  const dx = x - bb.cx;
  const dy = y - bb.cy;
  const localX = dx * cos + dy * aspect * sin;
  const localY = -dx / aspect * sin + dy * cos;
  const halfW = bb.width / 2;
  const halfH = bb.height / 2;
  return Math.abs(localX) <= halfW && Math.abs(localY) <= halfH;
}

/** 将关键点 (x, y) 约束到旋转包围框 bb 内，返回归一化坐标 */
function clampKpToBox(x: number, y: number, bb: any): { x: number; y: number } {
  if (!bb) return { x, y };
  const aspect = ch.value / cw.value;
  const cos = Math.cos(bb.angle);
  const sin = Math.sin(bb.angle);
  const dx = x - bb.cx;
  const dy = y - bb.cy;
  const localX = Math.max(-bb.width / 2, Math.min(bb.width / 2, dx * cos + dy * aspect * sin));
  const localY = Math.max(-bb.height / 2, Math.min(bb.height / 2, -dx / aspect * sin + dy * cos));
  return {
    x: bb.cx + localX * cos - localY / aspect * sin,
    y: bb.cy + localX * aspect * sin + localY * cos,
  };
}

/** 根据关键点数组计算旋转包围框（保持当前框的角度，只更新尺寸和中心） */
function fitBoundingBox(keypoints: any[], bb: any): { cx: number; cy: number; width: number; height: number; angle: number } {
  const xs = keypoints.map((kp: any) => kp.x);
  const ys = keypoints.map((kp: any) => kp.y);
  const x1 = Math.min(...xs), x2 = Math.max(...xs);
  const y1 = Math.min(...ys), y2 = Math.max(...ys);
  const cx = (x1 + x2) / 2;
  const cy = (y1 + y2) / 2;
  const w = x2 - x1, h = y2 - y1;
  return { cx, cy, width: Math.max(0.001, w), height: Math.max(0.001, h), angle: bb.angle };
}

/** 关键点颜色：优先使用 per-keypoint 自定义颜色，否则按序号回退渐变色 */
function kpColorByIndex(index: number, _classCount: number, classId?: number): string {
  if (classId !== undefined) {
    const cls = store.classes[classId];
    const colors = cls?.keypoint_colors;
    if (colors && colors.length > index) return colors[index];
  }
  const KP_COLORS = [
    "#FF6B6B", // 1  红
    "#FF9F43", // 2  橙
    "#FECA57", // 3  黄
    "#9CCC65", // 4  浅绿
    "#26DE81", // 5  绿
    "#20BF6B", // 6  深绿
    "#0BE881", // 7  青绿
    "#0FB9B1", // 8  青
    "#12CBC4", // 9  蓝绿
    "#0ABDE3", // 10 蓝
    "#2E86DE", // 11 深蓝
    "#3863D4", // 12 靛蓝
    "#8854D0", // 13 紫
    "#A55EEA", // 14 浅紫
    "#D980FA", // 15 粉紫
    "#F78FB3", // 16 粉
    "#EE5A70", // 17 深粉
  ];
  return KP_COLORS[index % KP_COLORS.length];
}

function keypointLabelLayout(ann: KeypointAnnotation) {
  const bb = ann.bounding_box;
  if (!bb) return null;
  const cos = Math.cos(-bb.angle);
  const sin = Math.sin(-bb.angle);
  // 矩形框四个角点在世界坐标系的位置（绕中心旋转）
  const corners = [
    { lx: -bb.width / 2, ly: -bb.height / 2 },
    { lx:  bb.width / 2, ly: -bb.height / 2 },
    { lx:  bb.width / 2, ly:  bb.height / 2 },
    { lx: -bb.width / 2, ly:  bb.height / 2 },
  ].map(c => ({
    x: bb.cx + c.lx * cos - c.ly * sin,
    y: bb.cy + c.lx * sin + c.ly * cos,
  }));
  const minX = Math.min(...corners.map(c => c.x * cw.value));
  const minY = Math.min(...corners.map(c => c.y * ch.value));
  const w = labelWidthForClass(ann.class_id);
  return {
    x: minX,
    y: minY - LABEL_TAG_H,
    w,
    tx: minX + w / 2,
    ty: minY - LABEL_TAG_H / 2,
  };
}

// ==================== OCR 辅助 ====================

function ocrTextPreviewVisible(_ann: OcrAnnotation): boolean {
  return true;
}

function ocrBBox(ann: OcrAnnotation) {
  if (!ann.points || ann.points.length < 2) return { minX: 0, minY: 0, maxX: 0, maxY: 0 };
  const xs = ann.points.map(p => p.x * cw.value);
  const ys = ann.points.map(p => p.y * ch.value);
  return { minX: Math.min(...xs), minY: Math.min(...ys), maxX: Math.max(...xs), maxY: Math.max(...ys) };
}

function ocrHandlePos(ann: OcrAnnotation, h: string) {
  const bb = ocrBBox(ann);
  const cx = (bb.minX + bb.maxX) / 2;
  const cy = (bb.minY + bb.maxY) / 2;
  if (h === "tl") return { x: bb.minX, y: bb.minY };
  if (h === "tr") return { x: bb.maxX, y: bb.minY };
  if (h === "bl") return { x: bb.minX, y: bb.maxY };
  if (h === "br") return { x: bb.maxX, y: bb.maxY };
  return { x: cx, y: cy };
}

function ocrCenter(ann: OcrAnnotation) {
  const bb = ocrBBox(ann);
  return { x: (bb.minX + bb.maxX) / 2, y: (bb.minY + bb.maxY) / 2 };
}

/** 判断 OCR 标注是否为矩形模式（4 个点构成轴对齐矩形） */
function isOcrRectMode(ann: OcrAnnotation): boolean {
  if (!ann.points || ann.points.length !== 4) return false;
  const eps = 0.001; // 归一化坐标容差
  const p = ann.points;
  // 矩形模式点顺序：TL(x1,y1) TR(x2,y1) BR(x2,y2) BL(x1,y2)
  return Math.abs(p[0].y - p[1].y) < eps && // 顶边水平
         Math.abs(p[1].x - p[2].x) < eps && // 右边垂直
         Math.abs(p[2].y - p[3].y) < eps && // 底边水平
         Math.abs(p[0].x - p[3].x) < eps;   // 左边垂直
}

function ocrLabelLayout(ann: OcrAnnotation): { x: number; y: number; w: number; tx: number; ty: number } {
  if (!ann.points || ann.points.length < 3) return { x: 0, y: 0, w: 0, tx: 0, ty: 0 };
  const xs = ann.points.map(p => p.x * cw.value);
  const ys = ann.points.map(p => p.y * ch.value);
  const minX = Math.min(...xs);
  const minY = Math.min(...ys);
  const text = ann.text || getClassName(ann.class_id);
  const w = Math.max(20, textPixelWidth(text) + 8);
  return {
    x: minX,
    y: minY - LABEL_TAG_H,
    w,
    tx: minX + w / 2,
    ty: minY - LABEL_TAG_H / 2,
  };
}

</script>

<style scoped>
.canvas-container {
  position: relative;
  width: 100%;
  height: 100%;
  background: #141414;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
}

.canvas-container.has-image {
  background: #d8d8d8;
}

.canvas-container.is-dragging {
  cursor: move;
}
.canvas-container.is-dragging * {
  cursor: inherit !important;
}
.canvas-container.hole-mode {
  box-shadow: inset 0 0 0 2px var(--accent);
}
.canvas-container.hole-mode::after {
  content: "🕳 孔洞模式 — 绘制多边形将为当前选中标注挖孔，按 H 退出";
  position: absolute;
  top: 8px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 20;
  padding: 4px 14px;
  background: var(--accent);
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  border-radius: 6px;
  pointer-events: none;
  white-space: nowrap;
}
.ocr-indicator {
  position: absolute;
  top: 8px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 20;
  padding: 4px 14px;
  background: var(--accent);
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  border-radius: 6px;
  pointer-events: none;
  white-space: nowrap;
}
.canvas-image {
  position: absolute;
  top: 50%;
  left: 50%;
  transform-origin: center center;
  display: block;
  pointer-events: none;
}
.annotation-svg {
  position: absolute;
  top: 50%;
  left: 50%;
  transform-origin: center center;
  overflow: visible;
}

.annotation-svg :deep(.ann-label) {
  pointer-events: none;
}

/* ---- 加载遮罩 ---- */
.crosshair {
  position: absolute;
  pointer-events: none;
  z-index: 15;
}
.crosshair-x {
  height: 1px;
  background: var(--accent);
  opacity: 0.5;
}
.crosshair-y {
  width: 1px;
  background: var(--accent);
  opacity: 0.5;
}
.loading-overlay {
  position: absolute;
  inset: 0;
  z-index: 20;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  background: rgba(20, 20, 20, 0.85);
  backdrop-filter: blur(4px);
  pointer-events: none;
}

.loading-spinner {
  width: 36px;
  height: 36px;
  border: 3px solid rgba(249, 115, 22, 0.2);
  border-top-color: #f97316;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-text {
  font-size: 13px;
  color: #a1a1aa;
  font-weight: 500;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 1;
}
.drop-zone {
  width: 420px;
  padding: 32px;
  border: 2px dashed var(--border-subtle);
  border-radius: 10px;
  background: #1a1a1a;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  text-align: center;
  cursor: pointer;
  outline: none;
  transition: border-color 0.15s, background 0.15s;
}
.drop-zone:hover {
  border-color: var(--accent);
  background: #1e1e1e;
}
.drop-zone:focus-visible {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(249, 115, 22, 0.35);
}
.drop-zone .icon { font-size: 40px; }
.drop-zone .title { font-size: 15px; color: var(--text-primary); }
.drop-zone .sub { font-size: 12px; color: var(--text-dim); }
.drop-zone .formats { font-size: 11px; color: var(--text-dim); }

/* 轴对齐矩形：角点斜向光标 */
.handle-corner-tl,
.handle-corner-br {
  cursor: nwse-resize;
}
.handle-corner-tr,
.handle-corner-bl {
  cursor: nesw-resize;
}
/* 边：上下 ns，左右 ew */
.handle-edge-t,
.handle-edge-b {
  cursor: ns-resize;
}
.handle-edge-l,
.handle-edge-r {
  cursor: ew-resize;
}

/* 旋转矩形：按方位光标 */
.handle-rb-tl,
.handle-rb-br {
  cursor: nwse-resize;
}
.handle-rb-tr,
.handle-rb-bl {
  cursor: nesw-resize;
}
.handle-rb-tc,
.handle-rb-bc {
  cursor: ns-resize;
}
.handle-rb-ml,
.handle-rb-mr {
  cursor: ew-resize;
}
.handle-rb-rotate {
  cursor: grab;
}

.move-handle {
  cursor: move;
}

/* 关键点包围框 */
.kp-bbox {
  cursor: move;
}
/* 关键点角点手柄 */
.kp-kp-vertex {
  cursor: default;
}
/* 关键点包围框手柄方位光标 */
.kp-kp-tl,
.kp-kp-br {
  cursor: nwse-resize;
}
.kp-kp-tr,
.kp-kp-bl {
  cursor: nesw-resize;
}
.kp-kp-tc,
.kp-kp-bc {
  cursor: ns-resize;
}
.kp-kp-ml,
.kp-kp-mr {
  cursor: ew-resize;
}
.handle-kp-vertex {
  cursor: default;
}

.handle-poly-vertex {
  cursor: default;
}

/* OCR 文本输入弹窗 */
.ocr-text-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.ocr-text-modal {
  background: #252526;
  border: 1px solid #3e3e42;
  border-radius: 8px;
  padding: 20px 24px;
  min-width: 320px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
}

.ocr-text-title {
  font-size: 14px;
  font-weight: 600;
  color: #e4e4e7;
  margin-bottom: 12px;
}

.ocr-text-input {
  width: 100%;
  padding: 8px 12px;
  background: #1e1e1e;
  border: 1px solid #3e3e42;
  border-radius: 6px;
  color: #e4e4e7;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
  margin-bottom: 12px;
  transition: border-color 0.15s;
}

.ocr-text-input:focus {
  border-color: #f97316;
}

.ocr-text-input::placeholder {
  color: #71717a;
}

.ocr-text-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.ocr-btn {
  padding: 6px 16px;
  border-radius: 5px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: background 0.15s;
}

.ocr-btn-cancel {
  background: #37373d;
  color: #a1a1aa;
}

.ocr-btn-cancel:hover {
  background: #3e3e42;
}

.ocr-btn-confirm {
  background: #f97316;
  color: #ffffff;
}

.ocr-btn-confirm:hover {
  background: #fb923c;
}
</style>
