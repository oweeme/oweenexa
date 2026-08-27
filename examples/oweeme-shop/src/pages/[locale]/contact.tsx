export const seo = {
    title: t("contact.title"),
    canonical: `/${params.locale}/contact`
};

export default function Contact() {
    return (
        <main>
            <h1>{t("contact.title")}</h1>
            <form data-nexa-form action="/api/contact" method="post">
                <label for="email">{t("contact.email")}</label>
                <input class="nx-input" id="email" name="email" type="email" required />
                <p data-nexa-error-for="email"></p>

                <label for="message">{t("contact.message")}</label>
                <input class="nx-input" id="message" name="message" type="text" required minlength="10" />
                <p data-nexa-error-for="message"></p>

                <button class="nx-btn nx-btn-primary" type="submit">{t("contact.send")}</button>
            </form>
        </main>
    );
}
