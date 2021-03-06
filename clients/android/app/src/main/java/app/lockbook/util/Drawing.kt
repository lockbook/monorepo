package app.lockbook.util

import android.graphics.Color
import com.beust.klaxon.Json
import java.util.LinkedHashMap

data class Drawing(
    var scale: Float = 1f,
    var translationX: Float = 0f,
    var translationY: Float = 0f,
    val strokes: MutableList<Stroke> = mutableListOf(),
    val theme: LinkedHashMap<String, ColorRGB> = linkedMapOf(
        Pair(ColorAlias.White.name, ColorRGB(0xFF, 0xFF, 0xFF)),
        Pair(ColorAlias.Black.name, ColorRGB(0x88, 0x88, 0x88)),
        Pair(ColorAlias.Red.name, ColorRGB(0xFF, 0x00, 0x00)),
        Pair(ColorAlias.Green.name, ColorRGB(0x00, 0xFF, 0x00)),
        Pair(ColorAlias.Yellow.name, ColorRGB(0xFF, 0xFF, 0x00)),
        Pair(ColorAlias.Blue.name, ColorRGB(0x00, 0x00, 0xFF)),
        Pair(ColorAlias.Magenta.name, ColorRGB(0xFF, 0x00, 0xFF)),
        Pair(ColorAlias.Cyan.name, ColorRGB(0x00, 0xFF, 0xFF)),
    )
) {
    fun getARGBColor(colorAlias: ColorAlias, alpha: Int): Int {
        val colorRGB = theme[colorAlias.name]!!
        return Color.argb(alpha, colorRGB.r, colorRGB.g, colorRGB.b)
    }

    fun themeToARGBColors(): LinkedHashMap<ColorAlias, Int> = linkedMapOf(
        Pair(ColorAlias.White, getARGBColor(ColorAlias.White, 255)),
        Pair(ColorAlias.Black, getARGBColor(ColorAlias.Black, 255)),
        Pair(ColorAlias.Red, getARGBColor(ColorAlias.Red, 255)),
        Pair(ColorAlias.Green, getARGBColor(ColorAlias.Green, 255)),
        Pair(ColorAlias.Yellow, getARGBColor(ColorAlias.Yellow, 255)),
        Pair(ColorAlias.Blue, getARGBColor(ColorAlias.Blue, 255)),
        Pair(ColorAlias.Magenta, getARGBColor(ColorAlias.Magenta, 255)),
        Pair(ColorAlias.Cyan, getARGBColor(ColorAlias.Cyan, 255)),
    )

    fun clone(): Drawing {
        return Drawing(
            scale,
            translationX,
            translationY,
            strokes.map { stroke ->
                Stroke(
                    stroke.pointsX.toMutableList(),
                    stroke.pointsY.toMutableList(),
                    stroke.pointsGirth.toMutableList(),
                    stroke.color,
                    stroke.alpha
                )
            }.toMutableList(),
            theme
        )
    }
}

data class Stroke(
    @Json(name = "points_x")
    val pointsX: MutableList<Float>,
    @Json(name = "points_y")
    val pointsY: MutableList<Float>,
    @Json(name = "points_girth")
    val pointsGirth: MutableList<Float>,
    val color: ColorAlias,
    val alpha: Int
)

enum class ColorAlias {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

data class ColorRGB(
    val r: Int,
    val g: Int,
    val b: Int,
)
