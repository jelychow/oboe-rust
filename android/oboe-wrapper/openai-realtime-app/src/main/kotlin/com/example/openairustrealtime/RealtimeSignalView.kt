package com.example.openairustrealtime

import android.content.Context
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.RectF
import android.view.View
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.max
import kotlin.math.sin

class RealtimeSignalView(context: Context) : View(context) {
    private val backgroundPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.rgb(18, 18, 16)
    }
    private val centerPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.rgb(62, 60, 54)
    }
    private val micPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.rgb(170, 219, 191)
    }
    private val outputPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.rgb(238, 204, 126)
    }
    private val labelPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        color = Color.rgb(166, 162, 153)
        textSize = dp(12).toFloat()
    }
    private val rect = RectF()
    private var micLevel = 0f
    private var outputLevel = 0f
    private var phase = 0f
    private var live = false

    init {
        minimumHeight = dp(132)
    }

    fun setLevels(micLevel: Float, outputLevel: Float) {
        this.micLevel = max(micLevel.coerceIn(0f, 1f), this.micLevel * 0.82f)
        this.outputLevel = max(outputLevel.coerceIn(0f, 1f), this.outputLevel * 0.82f)
        invalidate()
    }

    fun setLive(live: Boolean) {
        if (this.live != live) {
            this.live = live
            invalidate()
        }
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        val width = width.toFloat()
        val height = height.toFloat()
        val radius = dp(8).toFloat()
        rect.set(0f, 0f, width, height)
        canvas.drawRoundRect(rect, radius, radius, backgroundPaint)

        val centerY = height * 0.54f
        centerPaint.strokeWidth = dp(1).toFloat()
        canvas.drawLine(dp(18).toFloat(), centerY, width - dp(18), centerY, centerPaint)

        val usableWidth = width - dp(36)
        val step = usableWidth / BAR_COUNT
        val barWidth = max(dp(3).toFloat(), step * 0.42f)
        for (index in 0 until BAR_COUNT) {
            val normalized = index.toFloat() / max(1f, (BAR_COUNT - 1).toFloat())
            val x = dp(18) + index * step + (step - barWidth) / 2f
            val wave = sin(normalized * 8f + phase)
            val micHeight = dp(10) + dp(38) * micLevel * (0.55f + 0.45f * abs(wave))
            val outputHeight =
                dp(10) + dp(38) * outputLevel * (0.55f + 0.45f * abs(cos(wave + phase)))
            drawBar(canvas, micPaint, x, centerY - micHeight - dp(2), barWidth, micHeight)
            drawBar(canvas, outputPaint, x, centerY + dp(2), barWidth, outputHeight)
        }

        labelPaint.textAlign = Paint.Align.LEFT
        canvas.drawText("MIC", dp(18).toFloat(), dp(24).toFloat(), labelPaint)
        labelPaint.textAlign = Paint.Align.RIGHT
        canvas.drawText("ASSISTANT", width - dp(18), height - dp(18).toFloat(), labelPaint)

        phase += if (live) 0.22f else 0.04f
        postInvalidateDelayed(if (live) 33L else 120L)
    }

    private fun drawBar(
        canvas: Canvas,
        paint: Paint,
        x: Float,
        y: Float,
        width: Float,
        height: Float
    ) {
        rect.set(x, y, x + width, y + height)
        canvas.drawRoundRect(rect, width / 2f, width / 2f, paint)
    }

    private fun dp(value: Int): Int = (value * resources.displayMetrics.density).toInt()

    companion object {
        private const val BAR_COUNT = 28
    }
}
