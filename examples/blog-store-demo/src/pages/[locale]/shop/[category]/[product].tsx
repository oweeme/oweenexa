import Price from "../../../../components/Price";
export const load = { url: "/api/shop/:category/:product" };
export const paths = { url: "/api/shop" };
export const seo = { title: `${data.name} | ${t("shop.title")}`, description: data.name, canonical: `https://oweeme.com/${params.locale}/shop/${params.category}/${params.product}` };
export default function Product() {
    return (<article><h1>{data.name}</h1><Price amount={data.price} /><p id="tag">{t("common.tagline")}</p></article>);
}
